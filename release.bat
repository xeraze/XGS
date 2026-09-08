@echo off
setlocal

set DIST=%~dp0dist
set SRC=%~dp0

echo Building XGameStats...

if exist "%DIST%" rmdir /s /q "%DIST%"
mkdir "%DIST%"
mkdir "%DIST%\configs"
mkdir "%DIST%\gui\javafx-sdk\lib"

copy "%SRC%xgs.exe" "%DIST%\"
copy "%SRC%libscanner.dll" "%DIST%\"
copy "%SRC%libhooks.dll" "%DIST%\"
copy "%SRC%xgamestats-gui.jar" "%DIST%\"
copy "%SRC%configs\*.json" "%DIST%\configs\"
copy "%SRC%gui\javafx-sdk\lib\*.jar" "%DIST%\gui\javafx-sdk\lib\"

echo Done: %DIST%
echo Zip it: right-click dist/ -^> Send to -^> Compressed folder