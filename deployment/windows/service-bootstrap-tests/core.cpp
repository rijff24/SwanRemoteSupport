#include <windows.h>
#include <atomic>
#include <cstdint>

// This DLL tests the loader/service boundary only. It has no remote-control,
// network, company-profile, enrollment or authorization implementation.
namespace {
std::atomic<HANDLE> stop_event{nullptr};
}

BOOL WINAPI DllMain(HINSTANCE, DWORD reason, void*) {
#ifdef BOOTSTRAP_TEST_DELAY_MS
  if (reason == DLL_PROCESS_ATTACH) Sleep(BOOTSTRAP_TEST_DELAY_MS);
#endif
  return TRUE;
}

#ifndef BOOTSTRAP_TEST_MISSING_EXPORT
extern "C" __declspec(dllexport) uint32_t __cdecl swan_service_bootstrap(
    int(__cdecl* report)(uint32_t, uint32_t)) {
  HANDLE event = CreateEventW(nullptr, TRUE, FALSE, nullptr);
  if (!event) return GetLastError();
  stop_event.store(event, std::memory_order_release);
  if (!report(SERVICE_RUNNING, ERROR_SUCCESS)) return GetLastError();
  const DWORD wait = WaitForSingleObject(event, 60000);
  const DWORD result = wait == WAIT_OBJECT_0 ? ERROR_SUCCESS : ERROR_TIMEOUT;
  report(SERVICE_STOPPED, result);
  // Keep the event valid until process exit; a final control callback can race.
  return result;
}
#endif

extern "C" __declspec(dllexport) uint32_t __cdecl swan_service_control(uint32_t control) {
  if (control == SERVICE_CONTROL_INTERROGATE) return NO_ERROR;
  HANDLE event = stop_event.load(std::memory_order_acquire);
  if (control == SERVICE_CONTROL_STOP && event) {
    return SetEvent(event) ? NO_ERROR : GetLastError();
  }
  return ERROR_CALL_NOT_IMPLEMENTED;
}

// A core with only the legacy ABI must be rejected, rather than recursively
// attempting to connect a second service dispatcher.
extern "C" __declspec(dllexport) char** __cdecl rustdesk_core_main_args(int*) {
  return nullptr;
}
