@echo off
chcp 65001 >nul
setlocal EnableExtensions
title Sea Lantern 桌宠开发启动器
cd /d "%~dp0"

echo.
echo ==========================================
echo   Sea Lantern 桌宠开发启动器
echo ==========================================
echo.

rem 将 Rust 工具链加入当前窗口的 PATH，避免新终端找不到 cargo。
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

where node.exe >nul 2>nul
if errorlevel 1 (
  echo [错误] 未找到 Node.js，请先安装 Node.js 24 LTS。
  goto :failed
)

where pnpm.cmd >nul 2>nul
if errorlevel 1 (
  echo [错误] 未找到 pnpm，请先执行：
  echo        npm install -g pnpm@9.15.9
  goto :failed
)

where cargo.exe >nul 2>nul
if errorlevel 1 (
  echo [错误] 未找到 cargo，请先安装 Rust，或检查：
  echo        %USERPROFILE%\.cargo\bin
  goto :failed
)

rem 如果系统环境里已经有 cl.exe，就不再重复配置。
where cl.exe >nul 2>nul
if not errorlevel 1 goto :toolchain_ready

rem 当前机器已安装 Visual Studio 18 Community，但 VC 工具位于 ScopeCppSDK 目录。
set "VSROOT=%ProgramFiles%\Microsoft Visual Studio\18\Community"
set "VCROOT=%VSROOT%\SDK\ScopeCppSDK\vc15\VC"
set "SDKROOT=%VSROOT%\SDK\ScopeCppSDK\vc15\SDK"

if exist "%VCROOT%\bin\cl.exe" goto :configure_scope_sdk

rem 兜底：尝试用 vswhere 找到标准 VC 工具并加载 vcvars64.bat。
set "VSINSTALL="
if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" (
  for /f "usebackq delims=" %%I in (`"%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSINSTALL=%%I"
)

if defined VSINSTALL if exist "%VSINSTALL%\VC\Auxiliary\Build\vcvars64.bat" (
  echo [环境] 正在加载 Visual Studio C++ 构建环境...
  call "%VSINSTALL%\VC\Auxiliary\Build\vcvars64.bat" >nul
  goto :toolchain_ready
)

echo [错误] 未找到 MSVC 的 cl.exe。
echo        请安装 Visual Studio 的“使用 C++ 的桌面开发”工作负载，
echo        或把 cl.exe 所在目录加入 PATH 后重试。
goto :failed

:configure_scope_sdk
echo [环境] 正在加载 Visual Studio C++ 构建环境...
set "PATH=%VCROOT%\bin;%SDKROOT%\bin;%PATH%"
set "INCLUDE=%VCROOT%\include;%SDKROOT%\include;%SDKROOT%\include\shared;%SDKROOT%\include\ucrt;%SDKROOT%\include\um"
set "LIB=%VCROOT%\lib;%SDKROOT%\lib"

:toolchain_ready
echo [环境] Node.js:
node --version
echo [环境] pnpm:
call pnpm --version
echo [环境] Rust:
cargo --version
echo.

rem 如果本机 10800 代理正在监听，自动为本次下载/启动启用代理。
if not defined HTTP_PROXY (
  netstat -ano 2>nul | findstr ":10800" | findstr "LISTENING" >nul
  if not errorlevel 1 (
    set "HTTP_PROXY=http://127.0.0.1:10800"
    set "HTTPS_PROXY=http://127.0.0.1:10800"
    set "ALL_PROXY=http://127.0.0.1:10800"
    echo [环境] 已检测到本地代理 127.0.0.1:10800，已为本次启动启用。
  )
)
echo.

if /I "%~1"=="--check" (
  echo [完成] 开发环境检查通过，可以运行“启动.bat”启动桌宠。
  echo.
  pause
  exit /b 0
)

if not exist "node_modules\" (
  echo [依赖] 首次启动，正在安装前端依赖...
  call pnpm install
  if errorlevel 1 goto :failed
  echo.
)

echo [环境] 正在清理本项目残留的桌宠/Vite 进程...
powershell -NoProfile -ExecutionPolicy Bypass -Command "Get-CimInstance Win32_Process | Where-Object { $_.Name -in @('node.exe','sealantern.exe','cargo.exe','rustc.exe') -and $_.CommandLine -and $_.CommandLine.Contains((Get-Location).Path) } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }" >nul 2>nul
ping 127.0.0.1 -n 2 >nul
netstat -ano 2>nul | findstr ":5173" | findstr "LISTENING" >nul
if not errorlevel 1 (
  echo [错误] 端口 5173 仍被其他程序占用，请关闭占用该端口的程序后重试。
  goto :failed
)

echo [启动] 正在启动 Sea Lantern 和桌宠...
echo [提示] 桌宠默认出现在屏幕右下角；如果被隐藏，可在系统托盘中点“显示桌宠”。
echo [TIP] Press Ctrl+C to stop the dev server.
echo.

call pnpm tauri dev
if errorlevel 1 goto :failed

echo.
echo Sea Lantern 已退出。
pause
exit /b 0

:failed
echo.
echo 启动失败，请检查上方错误信息。
pause
exit /b 1
