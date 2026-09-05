cmake_minimum_required(VERSION 3.20)
project(GamePresenceHooks C)

set(CMAKE_C_STANDARD 17)
set(CMAKE_C_STANDARD_REQUIRED ON)

add_library(hooks SHARED hook.c)

target_include_directories(hooks PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})

if(MSVC)
    target_compile_options(hooks PRIVATE /W4)
else()
    target_compile_options(hooks PRIVATE -Wall -Wextra -pedantic)
endif()

set_target_properties(hooks PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin"
    ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
)

install(TARGETS hooks RUNTIME DESTINATION bin LIBRARY DESTINATION lib)
