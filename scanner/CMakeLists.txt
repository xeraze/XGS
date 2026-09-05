cmake_minimum_required(VERSION 3.20)
project(GamePresenceScanner C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(scanner SHARED scanner.cpp)

target_include_directories(scanner PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

if(MSVC)
    target_compile_options(scanner PRIVATE /W4)
else()
    target_compile_options(scanner PRIVATE -Wall -Wextra -pedantic)
endif()

set_target_properties(scanner PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
    ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
)

install(TARGETS scanner RUNTIME DESTINATION bin LIBRARY DESTINATION lib)