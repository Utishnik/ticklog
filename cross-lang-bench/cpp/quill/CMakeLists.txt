cmake_minimum_required(VERSION 3.20)

project(quill_harness LANGUAGES CXX)

# Quill is header-only and requires C++17.
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Host-native, maximum-performance build. quill compiles fine with
# exceptions disabled; keep RTTI off for speed - quill builds pass
# QUILL_NO_EXCEPTIONS and never throw.
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -march=native -flto -fno-exceptions -fno-rtti -fomit-frame-pointer")
set(CMAKE_CXX_FLAGS_RELEASE "-O3 -DNDEBUG")
set(CMAKE_CXX_FLAGS_DEBUG "-O0 -g")

# quill's CMakeLists is fetched from GitHub by default. For air-gapped
# / reconfigure-safe runs a full vendored checkout already lives in
# vendor/quill; when present we build from it directly (no network).
set(QUILL_VENDOR_DIR "${CMAKE_CURRENT_SOURCE_DIR}/vendor/quill")
if(EXISTS "${QUILL_VENDOR_DIR}/CMakeLists.txt")
    message(STATUS "using vendored quill at ${QUILL_VENDOR_DIR}")
    add_subdirectory("${QUILL_VENDOR_DIR}" "${CMAKE_CURRENT_BINARY_DIR}/_vendor/quill")
else()
    include(FetchContent)
    FetchContent_Declare(
        quill
        GIT_REPOSITORY https://github.com/odygrd/quill.git
        GIT_TAG        v12.1.0
        GIT_SHALLOW    TRUE
    )
    FetchContent_MakeAvailable(quill)
endif()

add_executable(quill_harness main.cpp)
target_compile_definitions(quill_harness PRIVATE QUILL_NO_EXCEPTIONS)
target_link_libraries(quill_harness PRIVATE quill::quill)

# Per-thread SPSC queue capacity in bytes (initial == unbounded max).
# 64 MiB targets Windows RAM; WSL2 builds pass 33554432.
if(NOT DEFINED QUILL_BENCH_QUEUE_CAPACITY)
    set(QUILL_BENCH_QUEUE_CAPACITY 67108864)
endif()
target_compile_definitions(quill_harness PRIVATE QUILL_BENCH_QUEUE_CAPACITY=${QUILL_BENCH_QUEUE_CAPACITY})
