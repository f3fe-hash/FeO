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

add_custom_target(runtime ALL
    COMMAND cargo build --release
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/runtime
    DEPENDS feo_core
)

add_custom_command(TARGET runtime POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CMAKE_SOURCE_DIR}/runtime/target/release/feo_runtime
        ${CMAKE_BINARY_DIR}/feo_runtime
)