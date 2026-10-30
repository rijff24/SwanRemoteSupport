#include "service_bootstrap.h"

#include <windows.h>
#include <shellapi.h>

#include <atomic>
#include <cstdint>
#include <string>
#include <vector>

namespace {
#ifndef SWAN_SERVICE_BOOTSTRAP_NAME
#define SWAN_SERVICE_BOOTSTRAP_NAME L"Swan Remote Support"
#endif
constexpr wchar_t kServiceName[] = SWAN_SERVICE_BOOTSTRAP_NAME;
constexpr DWORD kCheckpointIntervalMs = 5000;
#ifndef SWAN_SERVICE_BOOTSTRAP_TIMEOUT_MS
#define SWAN_SERVICE_BOOTSTRAP_TIMEOUT_MS 120000
#endif
constexpr DWORD kInitializationTimeoutMs = SWAN_SERVICE_BOOTSTRAP_TIMEOUT_MS;
using ReportStatus = int(__cdecl*)(uint32_t, uint32_t);
using CoreServiceMain = uint32_t(__cdecl*)(ReportStatus);
using CoreServiceControl = uint32_t(__cdecl*)(uint32_t);

SERVICE_STATUS_HANDLE status_handle = nullptr;
SERVICE_STATUS status = {};
SRWLOCK status_lock = SRWLOCK_INIT;
HANDLE initialization_finished = nullptr;
HANDLE heartbeat_ready = nullptr;
std::atomic<CoreServiceControl> core_control{nullptr};
std::atomic<DWORD> service_exit{ERROR_SUCCESS};

int __cdecl ReportCoreStatus(uint32_t state, uint32_t error) {
  if (state != SERVICE_RUNNING && state != SERVICE_STOPPED) {
    SetLastError(ERROR_INVALID_PARAMETER);
    return FALSE;
  }
  AcquireSRWLockExclusive(&status_lock);
  if (status.dwCurrentState == SERVICE_STOPPED) {
    ReleaseSRWLockExclusive(&status_lock);
    SetLastError(ERROR_SERVICE_NOT_ACTIVE);
    return FALSE;
  }
  status.dwCurrentState = state;
  status.dwControlsAccepted = state == SERVICE_RUNNING ? SERVICE_ACCEPT_STOP : 0;
  status.dwWin32ExitCode = error;
  status.dwCheckPoint = 0;
  status.dwWaitHint = 0;
  service_exit = error;
  const BOOL result = SetServiceStatus(status_handle, &status);
  SetEvent(initialization_finished);
  ReleaseSRWLockExclusive(&status_lock);
  return result;
}

DWORD WINAPI ControlHandler(DWORD control, DWORD, void*, void*) {
  auto handler = core_control.load(std::memory_order_acquire);
  if (handler) return handler(control);
  return control == SERVICE_CONTROL_INTERROGATE ? NO_ERROR
                                               : ERROR_CALL_NOT_IMPLEMENTED;
}

DWORD WINAPI InitializationHeartbeat(void*) {
  // Start this thread before LoadLibrary: a DLL's initialization may hold the
  // loader lock, which also gates a newly created thread's DLL notifications.
  SetEvent(heartbeat_ready);
  const ULONGLONG started = GetTickCount64();
  while (WaitForSingleObject(initialization_finished, kCheckpointIntervalMs) ==
         WAIT_TIMEOUT) {
    AcquireSRWLockExclusive(&status_lock);
    if (status.dwCurrentState != SERVICE_START_PENDING) {
      ReleaseSRWLockExclusive(&status_lock);
      return 0;
    }
    if (GetTickCount64() - started >= kInitializationTimeoutMs) {
      status.dwCurrentState = SERVICE_STOPPED;
      status.dwWin32ExitCode = ERROR_TIMEOUT;
      status.dwCheckPoint = 0;
      status.dwWaitHint = 0;
      service_exit = ERROR_TIMEOUT;
      SetServiceStatus(status_handle, &status);
      ReleaseSRWLockExclusive(&status_lock);
      // Initialization has not reached Running or accepted sessions. Do not
      // run DLL detach handlers while another thread may hold the loader lock.
      TerminateProcess(GetCurrentProcess(), ERROR_TIMEOUT);
      return ERROR_TIMEOUT;
    }
    ++status.dwCheckPoint;
    if (!SetServiceStatus(status_handle, &status)) {
      const DWORD error = GetLastError();
      service_exit = error;
      ReleaseSRWLockExclusive(&status_lock);
      TerminateProcess(GetCurrentProcess(), error);
      return error;
    }
    ReleaseSRWLockExclusive(&status_lock);
  }
  return 0;
}

HMODULE LoadServiceCore() {
  std::vector<wchar_t> path(32768);
  const DWORD length = GetModuleFileNameW(nullptr, path.data(),
                                        static_cast<DWORD>(path.size()));
  if (length == 0 || length >= path.size()) return nullptr;
  std::wstring directory(path.data(), length);
  const auto separator = directory.find_last_of(L"\\/");
  if (separator == std::wstring::npos) {
    SetLastError(ERROR_BAD_PATHNAME);
    return nullptr;
  }
  directory.resize(separator + 1);
  directory += L"librustdesk.dll";
  return LoadLibraryExW(directory.c_str(), nullptr,
                        LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR |
                            LOAD_LIBRARY_SEARCH_SYSTEM32);
}

void WINAPI ServiceMain(DWORD, wchar_t**) {
  status.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
  status.dwCurrentState = SERVICE_START_PENDING;
  status.dwCheckPoint = 1;
  status.dwWaitHint = 15000;
  status_handle = RegisterServiceCtrlHandlerExW(kServiceName, ControlHandler,
                                               nullptr);
  if (!status_handle) {
    service_exit = GetLastError();
    return;
  }
  if (!SetServiceStatus(status_handle, &status)) {
    service_exit = GetLastError();
    return;
  }
  initialization_finished = CreateEventW(nullptr, TRUE, FALSE, nullptr);
  DWORD event_error = initialization_finished ? ERROR_SUCCESS : GetLastError();
  heartbeat_ready = CreateEventW(nullptr, TRUE, FALSE, nullptr);
  if (!heartbeat_ready) event_error = GetLastError();
  if (!initialization_finished || !heartbeat_ready) {
    // Report directly because ReportCoreStatus also signals the event.
    status.dwCurrentState = SERVICE_STOPPED;
    status.dwWin32ExitCode = event_error;
    status.dwCheckPoint = 0;
    status.dwWaitHint = 0;
    service_exit = status.dwWin32ExitCode;
    SetServiceStatus(status_handle, &status);
    if (initialization_finished) CloseHandle(initialization_finished);
    if (heartbeat_ready) CloseHandle(heartbeat_ready);
    return;
  }
  HANDLE heartbeat = CreateThread(nullptr, 0, InitializationHeartbeat, nullptr,
                                  0, nullptr);
  if (!heartbeat) {
    ReportCoreStatus(SERVICE_STOPPED, GetLastError());
  } else if (WaitForSingleObject(heartbeat_ready, 10000) != WAIT_OBJECT_0) {
    ReportCoreStatus(SERVICE_STOPPED, ERROR_TIMEOUT);
  } else {
    HMODULE core = LoadServiceCore();
    if (!core) {
      ReportCoreStatus(SERVICE_STOPPED, GetLastError());
    } else {
      auto main = reinterpret_cast<CoreServiceMain>(
          GetProcAddress(core, "swan_service_bootstrap"));
      auto control = reinterpret_cast<CoreServiceControl>(
          GetProcAddress(core, "swan_service_control"));
      if (!main || !control) {
        ReportCoreStatus(SERVICE_STOPPED, ERROR_PROC_NOT_FOUND);
      } else {
        core_control.store(control, std::memory_order_release);
        const DWORD result = main(ReportCoreStatus);
        ReportCoreStatus(SERVICE_STOPPED, result);
        core_control.store(nullptr, std::memory_order_release);
      }
      // Keep the core loaded until process exit: a final control callback can
      // race shutdown. No legacy core entry point is used as a fallback.
    }
  }
  SetEvent(initialization_finished);
  if (heartbeat) {
    if (WaitForSingleObject(heartbeat, 5000) != WAIT_OBJECT_0) {
      TerminateProcess(GetCurrentProcess(), ERROR_TIMEOUT);
    }
    CloseHandle(heartbeat);
  }
  CloseHandle(heartbeat_ready);
  CloseHandle(initialization_finished);
}
}  // namespace

bool RunSwanServiceIfRequested(int* exit_code) {
  int count = 0;
  auto args = CommandLineToArgvW(GetCommandLineW(), &count);
  if (!args) {
    *exit_code = static_cast<int>(GetLastError());
    return true;
  }
  bool service = false;
  for (int i = 1; i < count; ++i) {
    if (wcscmp(args[i], L"--service") == 0) service = true;
  }
  LocalFree(args);
  if (!service) return false;
  if (count != 2) {
    *exit_code = ERROR_INVALID_PARAMETER;
    return true;
  }
  SERVICE_TABLE_ENTRYW table[] = {
      {const_cast<wchar_t*>(kServiceName), ServiceMain}, {nullptr, nullptr}};
  if (!StartServiceCtrlDispatcherW(table)) {
    *exit_code = static_cast<int>(GetLastError());
  } else {
    *exit_code = static_cast<int>(service_exit.load(std::memory_order_acquire));
  }
  return true;
}
