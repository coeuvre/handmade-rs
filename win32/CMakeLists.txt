add_executable(win32_handmade WIN32 src/win32_handmade.cpp)
target_compile_options(win32_handmade PRIVATE
        /FC)
target_compile_definitions(win32_handmade PRIVATE
        _CRT_SECURE_NO_WARNINGS
        HANDMADE_INTERNAL)
target_link_libraries(win32_handmade
        debug ${CMAKE_SOURCE_DIR}/target/debug/handmade.dll.lib)