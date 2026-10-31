#ifndef SWAN_SERVICE_BOOTSTRAP_H_
#define SWAN_SERVICE_BOOTSTRAP_H_

// Returns false for desktop invocations. A handled service invocation must
// return directly from wWinMain, without loading the desktop or Rust core again.
bool RunSwanServiceIfRequested(int* exit_code);

#endif
