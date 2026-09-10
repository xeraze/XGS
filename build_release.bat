@echo off
setlocal
set DIST_DIR=%~dp0dist
set CORE_DIR=%~dp0core
set GUI_DIR=%~dp0gui
set SCAN_DIR=%~dp0scanner
set HOOK_DIR=%~dp0hooks
set INSTALLER_DIR=%~dp0installer
set INSTALLER_RS=%~dp0installer-rs
set JAVAFX=%GUI_DIR%\javafx-sdk\lib
set MODS=javafx.controls,javafx.fxml

for /f "tokens=2 delims== " %%a in ('findstr "^version" "%CORE_DIR%\Cargo.toml"') do (
    set VERSION=%%~a
)
set VERSION=%VERSION:"=%

echo [XGS] Building XGameStats v%VERSION%...
echo.

echo [1/8] Building Rust core...
cd /d "%CORE_DIR%"
cargo +stable-x86_64-pc-windows-gnu build --release
if %ERRORLEVEL% neq 0 (echo [ERROR] Rust failed & exit /b 1)
echo [OK] xgs.exe
echo.

echo [2/8] Building C++ scanner...
cd /d "%SCAN_DIR%"
if not exist build\CMakeCache.txt (
    cmake -S . -B build -G "MinGW Makefiles"
    if %ERRORLEVEL% neq 0 (echo [ERROR] scanner configure failed & exit /b 1)
)
cmake --build build
if %ERRORLEVEL% neq 0 (echo [ERROR] scanner build failed & exit /b 1)
echo [OK] libscanner.dll
echo.

echo [3/8] Building C++ hooks...
cd /d "%HOOK_DIR%"
if not exist build\CMakeCache.txt (
    cmake -S . -B build -G "MinGW Makefiles"
    if %ERRORLEVEL% neq 0 (echo [ERROR] hooks configure failed & exit /b 1)
)
cmake --build build --clean-first
if %ERRORLEVEL% neq 0 (echo [ERROR] hooks build failed & exit /b 1)
echo [OK] libhooks.dll
echo.
echo [4/8] Compiling Java GUI...
cd /d "%GUI_DIR%"
if exist build\com rmdir /s /q build\com
javac --module-path "%JAVAFX%" --add-modules %MODS% -d build src\main\java\com\gamepresence\gui\Main.java
if %ERRORLEVEL% neq 0 (echo [ERROR] GUI failed & exit /b 1)
rem Вкладываем dark-theme.css внутрь jar (ресурс ищется как /dark-theme.css)
copy /y src\main\resources\dark-theme.css build\com\dark-theme.css >nul
jar --create --file build\xgamestats-gui.jar --main-class com.gamepresence.gui.Main -C build com -C src\main\resources dark-theme.css
if %ERRORLEVEL% neq 0 (echo [ERROR] GUI jar failed & exit /b 1)
echo [OK] xgamestats-gui.jar
echo.

echo [5/8] Compiling Installer...
cd /d "%INSTALLER_DIR%"
if exist build\com rmdir /s /q build\com
javac --module-path "%JAVAFX%" --add-modules %MODS% -d build src\main\java\com\xgs\installer\Installer.java
if %ERRORLEVEL% neq 0 (echo [ERROR] Installer failed & exit /b 1)
jar --create --file build\xgs-installer.jar --main-class com.xgs.installer.Installer -C build com
echo [OK] xgs-installer.jar
echo.

echo [6/8] Building Rust installer...
cd /d "%INSTALLER_RS%"
cargo +stable-x86_64-pc-windows-gnu build --release
if %ERRORLEVEL% neq 0 (echo [ERROR] Rust installer failed & exit /b 1)
echo [OK] xgs-setup.exe
echo.

echo [7/8] Assembling distribution...
cd /d "%~dp0"
if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
mkdir "%DIST_DIR%"
mkdir "%DIST_DIR%\javafx-sdk\lib"
mkdir "%DIST_DIR%\configs"
mkdir "%DIST_DIR%\assets"

copy "%CORE_DIR%\target\x86_64-pc-windows-gnu\release\xgs.exe" "%DIST_DIR%\xgs.exe"
copy "%GUI_DIR%\build\xgamestats-gui.jar" "%DIST_DIR%\xgamestats-gui.jar"
copy "%INSTALLER_DIR%\build\xgs-installer.jar" "%DIST_DIR%\xgs-installer.jar"
copy "%INSTALLER_RS%\target\x86_64-pc-windows-gnu\release\xgs-installer.exe" "%DIST_DIR%\xgs-setup.exe"
copy "%SCAN_DIR%\build\bin\libscanner.dll" "%DIST_DIR%\libscanner.dll"
copy "%HOOK_DIR%\build\bin\libhooks.dll" "%DIST_DIR%\libhooks.dll"
copy "%~dp0translations.json" "%DIST_DIR%\translations.json"
copy "%~dp0version.json" "%DIST_DIR%\version.json"
copy "%~dp0README.md" "%DIST_DIR%\README.md"
copy "%~dp0SECURITY.md" "%DIST_DIR%\SECURITY.md"
copy "%GUI_DIR%\javafx-sdk\lib\*.jar" "%DIST_DIR%\javafx-sdk\lib\"
copy "%~dp0configs\*.json" "%DIST_DIR%\configs\"
copy "%~dp0assets\logo.png" "%DIST_DIR%\assets\logo.png"
copy "%~dp0assets\logo.ico" "%DIST_DIR%\assets\logo.ico"
echo [OK] dist/ ready
echo.

echo [8/8] Creating ZIP...
powershell -command "Compress-Archive -Path '%DIST_DIR%\*' -DestinationPath '%~dp0XGameStats-v%VERSION%.zip' -Force"
echo [OK] XGameStats-v%VERSION%.zip
echo.

echo ============================================
echo Build complete! v%VERSION%
echo ============================================
pause