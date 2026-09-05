@echo off
title XGameStats
color 0B
mode con: cols=50 lines=22
cls
echo.
echo   ██╗  ██╗     ███████╗ ███████╗  ██████╗
echo   ╚██╗██╔╝     ██╔════╝ ██╔════╝ ██╔════╝
echo    ╚███╔╝ █████╗███████╗ █████╗  ██║  ███╗
echo    ██╔██╗ ██╔══╝╚════██║ ██╔══╝  ██║   ██║
echo   ██╔╝ ██╗███████╗███████║ ███████╗╚██████╔╝
echo   ╚═╝  ╚═╝╚══════╝╚══════╝ ╚══════╝ ╚═════╝
echo        Discord Rich Presence for any game
echo.
echo   ─────────────────────────────────────────
echo.
echo    1)  Start XGameStats
echo    2)  Engine only (no GUI)
echo    3)  GUI only (no engine)
echo    4)  Exit
echo.
echo   ─────────────────────────────────────────
echo.

set /p choice="   > "

if "%choice%"=="1" (
    "%~dp0xgs.exe" start
) else if "%choice%"=="2" (
    "%~dp0xgs.exe" engine
) else if "%choice%"=="3" (
    "%~dp0xgs.exe" gui
) else (
    exit
)
