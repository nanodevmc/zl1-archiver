@echo off
chcp 65001 >nul
echo ========================================================
echo       УСТАНОВЩИК ИНТЕГРАЦИИ ZL1 ARCHIVER В СИСТЕМУ
echo ========================================================
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\ZL1Archiver"
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

copy /y "%~dp0zl1-cli.exe" "%INSTALL_DIR%\" >nul
copy /y "%~dp0zl1-gui.exe" "%INSTALL_DIR%\" >nul
if exist "%~dp0assets\icon.svg" copy /y "%~dp0assets\icon.svg" "%INSTALL_DIR%\" >nul

echo Регистрация пунктов в контекстном меню Windows...

:: 1. Меню для файлов (*): "Сжать в .zl1"
reg add "HKCU\Software\Classes\*\shell\ZL1Compress" /ve /t REG_SZ /d "Сжать в .zl1 (ZL1 Archiver)" /f >nul
reg add "HKCU\Software\Classes\*\shell\ZL1Compress\command" /ve /t REG_SZ /d "\"%INSTALL_DIR%\zl1-cli.exe\" c \"%%1\" \"%%1.zl1\"" /f >nul

:: 2. Меню для папок (Directory): "Сжать в .zl1"
reg add "HKCU\Software\Classes\Directory\shell\ZL1Compress" /ve /t REG_SZ /d "Сжать папку в .zl1 (ZL1 Archiver)" /f >nul
reg add "HKCU\Software\Classes\Directory\shell\ZL1Compress\command" /ve /t REG_SZ /d "\"%INSTALL_DIR%\zl1-cli.exe\" c \"%%1\" \"%%1.zl1\"" /f >nul

:: 3. Меню для архивов .zl1: "Распаковать .zl1"
reg add "HKCU\Software\Classes\.zl1" /ve /t REG_SZ /d "ZL1Archive" /f >nul
reg add "HKCU\Software\Classes\ZL1Archive\shell\open" /ve /t REG_SZ /d "Распаковать архив .zl1" /f >nul
reg add "HKCU\Software\Classes\ZL1Archive\shell\open\command" /ve /t REG_SZ /d "\"%INSTALL_DIR%\zl1-cli.exe\" x \"%%1\" \"%%1_extracted\"" /f >nul

echo.
echo [УСПЕХ] ZL1 Archiver успешно установлен в %INSTALL_DIR%!
echo Теперь при клике правой кнопкой мыши на любой файл или папку доступно:
echo "Сжать в .zl1 (ZL1 Archiver)"
echo.
pause
