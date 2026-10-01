@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
if errorlevel 1 exit /b 1
cd /d "%~dp0"
cl /nologo /O2 /W3 /D_CRT_SECURE_NO_WARNINGS profiler_cli.c /link dbghelp.lib psapi.lib
if errorlevel 1 exit /b 1
cl /nologo /O2 spin_test.c >nul 2>&1
if errorlevel 1 exit /b 1
spin_test.exe
exit /b %errorlevel%
