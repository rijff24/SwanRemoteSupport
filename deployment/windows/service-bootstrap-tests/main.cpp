#include <windows.h>
#include "service_bootstrap.h"

int WINAPI wWinMain(HINSTANCE, HINSTANCE, wchar_t*, int) {
  int code = 0;
  if (RunSwanServiceIfRequested(&code)) return code;
  return 7;  // The real desktop entry point is deliberately absent in this fixture.
}
