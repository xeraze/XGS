@echo off
chcp 65001 >nul
title XGameStats
color 0B
mode con: cols=62 lines=26
cls
echo.
echo                  ██╗  ██╗ ██████╗ ███████╗
echo                  ╚██╗██╔╝██╔════╝ ██╔════╝
echo                   ╚███╔╝ ██║  ███╗███████╗
echo                   ██╔██╗ ██║   ██║╚════██║
echo                  ██╔╝ ██╗╚██████╔╝███████║
echo                  ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
echo.
echo          ╔════════════════════════════════════════╗
echo          ║                 v0.1.0                 ║
echo          ╚════════════════════════════════════════╝
echo.
echo          ╔════════════════════════════════════════╗
echo          ║                                        ║
echo          ║  1)  Start XGameStats                  ║
echo          ║  2)  Engine only                       ║
echo          ║  3)  Exit                              ║
echo          ║                                        ║
echo          ╚════════════════════════════════════════╝
echo.

set /p choice="   > "

if "%choice%"=="1" (
    "%~dp0xgs.exe" start
) else if "%choice%"=="2" (
    "%~dp0xgs.exe" engine
) else (
    exit
)