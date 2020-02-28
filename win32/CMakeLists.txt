add_executable(win32_handmade WIN32 src/win32_handmade.cpp)
target_compile_options(win32_handmade PRIVATE
        /incremental:no /MTd /Gm- /Od /Oi /FC /W4 /WX /wd4201 /wd4100 /wd4189 /Fmwin32_handmade.map)
target_compile_definitions(win32_handmade PRIVATE
        _CRT_SECURE_NO_WARNINGS
        HANDMADE_INTERNAL=1
        HANDMADE_SLOW=1
        HANDMADE_WIN32=1)
target_link_options(win32_handmade PRIVATE /opt:ref)
target_link_libraries(win32_handmade winmm)