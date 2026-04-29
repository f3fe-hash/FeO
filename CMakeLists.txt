cmake_minimum_required(VERSION 3.16)
project(FeO C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 23)

include_directories(include lib)

file(GLOB_RECURSE SOURCES src/*.c src/*.cpp)

add_library(feo_core STATIC ${SOURCES})

find_package(OpenSSL REQUIRED)

target_link_libraries(feo_core PRIVATE OpenSSL::SSL OpenSSL::Crypto)

find_program(RUSTC rustc REQUIRED)

set(RUST_SRC ${CMAKE_SOURCE_DIR}/runtime/main.rs)

add_custom_command(
    OUTPUT feo_runtime
    COMMAND ${RUSTC} ${RUST_SRC}
            -L ${CMAKE_BINARY_DIR}
            -l static=feo_core
            -l ssl
            -l crypto
            -o feo_runtime
    DEPENDS feo_core
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
)

add_custom_target(runtime ALL DEPENDS feo_runtime)