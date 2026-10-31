cmake_minimum_required(VERSION 3.20)
project(SwanServiceBootstrapChecks LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
set(bootstrap "${CMAKE_CURRENT_SOURCE_DIR}/../../../flutter/windows/runner")

foreach(name bootstrap-check bootstrap-timeout-check)
  add_executable(${name} WIN32 main.cpp "${bootstrap}/service_bootstrap.cpp")
  target_include_directories(${name} PRIVATE "${bootstrap}")
  target_link_libraries(${name} PRIVATE advapi32 shell32)
  target_compile_definitions(${name} PRIVATE UNICODE _UNICODE)
endforeach()
target_compile_definitions(bootstrap-check PRIVATE
  "SWAN_SERVICE_BOOTSTRAP_NAME=L\"SwanLabBootstrapFixture\"")
target_compile_definitions(bootstrap-timeout-check PRIVATE
  "SWAN_SERVICE_BOOTSTRAP_NAME=L\"SwanLabBootstrapTimeoutFixture\""
  SWAN_SERVICE_BOOTSTRAP_TIMEOUT_MS=10000)

foreach(name core-fast core-slow core-timeout core-missing-export)
  add_library(${name} SHARED core.cpp)
endforeach()
target_compile_definitions(core-slow PRIVATE BOOTSTRAP_TEST_DELAY_MS=35000)
target_compile_definitions(core-timeout PRIVATE BOOTSTRAP_TEST_DELAY_MS=30000)
target_compile_definitions(core-missing-export PRIVATE BOOTSTRAP_TEST_MISSING_EXPORT)
