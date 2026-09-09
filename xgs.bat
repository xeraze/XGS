@echo off
chcp 65001 >nul
title XGameStats
mode con: cols=50 lines=24
cls
echo.
echo   XGameStats v0.6.0
echo   Discord Rich Presence Engine
echo.
echo   1)  Start XGameStats
echo   2)  Engine only
echo   3)  Exit
echo.

set /p choice="   > "

if "%choice%"=="1" (
    "%~dp0xgs.exe" start
) else if "%choice%"=="2" (
    "%~dp0xgs.exe" engine
) else (
    exit
)