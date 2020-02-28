@echo off

setlocal

cargo build

copy target\debug\handmade.dll build\win32\

if not exist "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvarsall.bat" (
    echo Please install Visual Studio 2019!
    goto :eof
)

call "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvarsall.bat" x64

cmake --build build\
