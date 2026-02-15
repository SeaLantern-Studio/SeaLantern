@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

:: 海晶灯发布脚本 (Windows 版本)
:: 用于自动更新版本号并推送到指定分支

echo.
echo ========================================
echo   海晶灯 (Sea Lantern) 发布工具
echo ========================================
echo.

:: 检查是否在 git 仓库中
git rev-parse --git-dir >nul 2>&1
if errorlevel 1 (
    echo [错误] 当前目录不是 Git 仓库
    pause
    exit /b 1
)

:: 检查工作区是否干净
git diff-index --quiet HEAD -- >nul 2>&1
if errorlevel 1 (
    echo [警告] 工作区有未提交的更改
    git status --short
    echo.
    set /p continue="是否继续？(y/N): "
    if /i not "!continue!"=="y" (
        echo 已取消
        pause
        exit /b 0
    )
)

:: 获取当前版本号
for /f "tokens=2 delims=:, " %%a in ('findstr /r "\"version\":" src-tauri\tauri.conf.json') do (
    set CURRENT_VERSION=%%~a
    goto :version_found
)
:version_found
echo [信息] 当前版本: %CURRENT_VERSION%
echo.

:: 输入新版本号
set /p NEW_VERSION="请输入新版本号 (例如: 0.6.0): "

:: 简单验证版本号格式
echo %NEW_VERSION% | findstr /r "^[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*$" >nul
if errorlevel 1 (
    echo [错误] 版本号格式不正确，应该是 x.y.z 格式 (例如: 0.6.0)
    pause
    exit /b 1
)

echo [信息] 新版本号: %NEW_VERSION%
echo.

:: 选择推送分支
echo 请选择推送目标：
echo   1) main    - 代码质量检查 (不构建安装包)
echo   2) release - 自动构建发布 (编译打包所有平台)
echo.
set /p BRANCH_CHOICE="请选择 (1/2): "

if "%BRANCH_CHOICE%"=="1" (
    set TARGET_BRANCH=main
    echo [信息] 目标分支: main (只检查代码质量)
) else if "%BRANCH_CHOICE%"=="2" (
    set TARGET_BRANCH=release
    echo [警告] 目标分支: release (将触发完整构建，需要 10-30 分钟)
) else (
    echo [错误] 无效的选择
    pause
    exit /b 1
)

echo.
echo [警告] 即将执行以下操作：
echo   1. 更新版本号: %CURRENT_VERSION% -^> %NEW_VERSION%
echo   2. 提交更改到本地仓库
echo   3. 推送到 GitHub 的 %TARGET_BRANCH% 分支
echo.
set /p confirm="确认继续？(y/N): "
if /i not "!confirm!"=="y" (
    echo 已取消
    pause
    exit /b 0
)

:: 更新 package.json 中的版本号
echo [信息] 更新 package.json...
powershell -Command "(Get-Content package.json) -replace '\"version\": \".*\"', '\"version\": \"%NEW_VERSION%\"' | Set-Content package.json"
echo [成功] 已更新 package.json

:: 更新 tauri.conf.json 中的版本号
echo [信息] 更新 src-tauri\tauri.conf.json...
powershell -Command "(Get-Content src-tauri\tauri.conf.json) -replace '\"version\": \".*\"', '\"version\": \"%NEW_VERSION%\"' | Set-Content src-tauri\tauri.conf.json"
echo [成功] 已更新 src-tauri\tauri.conf.json

:: 更新 Cargo.toml 中的版本号
echo [信息] 更新 src-tauri\Cargo.toml...
powershell -Command "(Get-Content src-tauri\Cargo.toml) -replace '^version = \".*\"', 'version = \"%NEW_VERSION%\"' | Set-Content src-tauri\Cargo.toml"
echo [成功] 已更新 src-tauri\Cargo.toml

:: 提交更改
echo [信息] 提交更改...
git add package.json src-tauri\tauri.conf.json src-tauri\Cargo.toml
git commit -m "chore: bump version to %NEW_VERSION%"
echo [成功] 已提交到本地仓库

:: 推送到 GitHub
echo [信息] 推送到 GitHub %TARGET_BRANCH% 分支...
git push github HEAD:%TARGET_BRANCH%
if errorlevel 1 (
    echo [错误] 推送失败
    echo [提示] 你可能需要手动推送: git push github HEAD:%TARGET_BRANCH%
    pause
    exit /b 1
)

echo [成功] 推送成功！
echo.

if "%TARGET_BRANCH%"=="release" (
    echo ========================================
    echo   🎉 发布流程已启动！
    echo ========================================
    echo.
    echo 接下来：
    echo   1. 访问 https://github.com/FPSZ/SeaLantern/actions
    echo   2. 等待构建完成 (大约 10-30 分钟)
    echo   3. 到 https://github.com/FPSZ/SeaLantern/releases 查看草稿
    echo   4. 编辑发布说明，点击 'Publish release' 正式发布
) else (
    echo ========================================
    echo   ✓ 代码质量检查已启动
    echo ========================================
    echo.
    echo 访问 https://github.com/FPSZ/SeaLantern/actions 查看检查结果
)

echo.
pause
