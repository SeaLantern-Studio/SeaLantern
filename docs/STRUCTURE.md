# Sea Lantern 项目结构

## 项目概览

Sea Lantern（海晶灯）是一个轻量化的 Minecraft 服务器管理工具，基于 Tauri 2 + Rust + Vue 3 技术栈。

## 详细结构

```
sea-lantern/
│
├── src/                                # 前端代码（Vue 3 + TypeScript）
│   │
│   ├── api/                           # 与 Rust 后端通信的封装层
│   │   ├── index.ts                   # API 入口
│   │   ├── tauri.ts                  # 基础 invoke 封装，所有 API 的入口
│   │   ├── server.ts                 # 服务器管理 API（创建、启动、停止、日志）
│   │   ├── java.ts                   # Java 环境检测 API
│   │   ├── config.ts                 # 配置文件读写 API
│   │   ├── player.ts                 # 玩家管理 API（白名单、封禁、OP）
│   │   ├── settings.ts               # 应用设置 API
│   │   ├── system.ts                 # 系统信息、文件对话框 API
│   │   ├── update.ts                 # 软件更新检查 API
│   │   ├── remoteLocales.ts          # 远程语言包 API
│   │   ├── plugin.ts                 # 插件系统 API
│   │   ├── mcs_plugins.ts            # MCS 插件 API
│   │   └── downloader.ts             # 下载管理 API
│   │
│   ├── assets/                        # 静态资源
│   │   └── logo.svg                  # 应用图标
│   │
│   ├── components/                    # UI 组件
│   │   ├── common/                   # 通用组件（整个项目的积木块）
│   │   │   ├── BrandIcon.vue         # 品牌图标组件
│   │   │   ├── SLButton.vue          # 按钮组件
│   │   │   ├── SLCard.vue            # 卡片容器
│   │   │   ├── SLInput.vue           # 输入框组件
│   │   │   ├── SLSelect.vue          # 下拉选择组件
│   │   │   ├── SLSwitch.vue          # 开关组件
│   │   │   ├── SLModal.vue           # 弹窗组件
│   │   │   ├── SLProgress.vue        # 进度条组件
│   │   │   ├── SLBadge.vue           # 状态标签组件
│   │   │   ├── SLCheckbox.vue        # 复选框组件
│   │   │   ├── SLFormField.vue       # 表单字段组件
│   │   │   ├── SLTextarea.vue        # 文本域组件
│   │   │   ├── SLTabs.vue            # 标签页组件
│   │   │   ├── SLTabBar.vue          # 标签栏组件
│   │   │   ├── SLSpinner.vue         # 加载动画组件
│   │   │   ├── SLContextMenu.vue     # 上下文菜单组件
│   │   │   ├── SLConfirmDialog.vue   # 确认对话框组件
│   │   │   ├── UpdateModal.vue       # 更新模态框组件
│   │   │   └── index.ts              # 组件导出文件
│   │   │
│   │   ├── config/                   # 配置相关组件
│   │   │   ├── ConfigCategories.vue  # 配置分类组件
│   │   │   ├── ConfigEntry.vue       # 配置项组件
│   │   │   └── ConfigToolbar.vue     # 配置工具栏组件
│   │   │
│   │   ├── console/                  # 控制台相关组件
│   │   │   ├── CommandModal.vue      # 命令模态框组件
│   │   │   ├── ConsoleInput.vue      # 控制台输入组件
│   │   │   ├── ConsoleOutput.vue     # 控制台输出组件
│   │   │
│   │   ├── layout/                   # 页面布局组件
│   │   │   ├── AppLayout.vue         # 总布局（左侧栏 + 右侧内容区）
│   │   │   ├── AppSidebar.vue        # 侧边导航栏
│   │   │   ├── AppHeader.vue         # 顶部标题栏
│   │   │   └── index.ts              # 布局组件导出文件
│   │   │
│   │   ├── plugin/                   # 插件相关组件
│   │   │   ├── PluginComponentRenderer.vue  # 插件组件渲染器
│   │   │   ├── PluginPermissionPanel.vue    # 插件权限面板
│   │   │   ├── SLPermissionDialog.vue       # 权限对话框
│   │   │   └── index.ts              # 插件组件导出文件
│   │   │
│   │   ├── splash/                   # 启动画面
│   │   │   └── SplashScreen.vue      # 应用启动时的加载动画
│   │   │
│   │   ├── views/                    # 页面相关子组件
│   │   │   ├── about/                # 关于页面组件
│   │   │   ├── create/               # 创建服务器页面组件
│   │   │   ├── home/                 # 首页组件
│   │   │   ├── paint/                # 个性化设置页面组件
│   │   │   ├── player/               # 玩家管理页面组件
│   │   │   └── settings/             # 设置页面组件
│   │   │
│   │   ├── JavaDownloader.vue        # Java 下载器组件
│   │   └── index.ts                  # 组件导出文件
│   │
│   ├── composables/                   # 组合式函数
│   │   ├── useAsync.ts               # 异步操作处理
│   │   ├── useComponentRegistry.ts   # 组件注册表
│   │   ├── useMessage.ts             # 消息处理
│   │   ├── useRegisterComponent.ts   # 组件注册函数
│   │   ├── useTabIndicator.ts        # 标签指示器
│   │   ├── useToast.ts               # 提示组件的组合式函数
│   │   └── useAboutLinks.ts          # 关于页面链接处理
│   │
│   ├── data/                          # 静态数据
│   │   └── contributors.ts           # 贡献者信息列表
│   │
│   ├── language/                      # 国际化资源
│   │   ├── index.ts                   # i18n 核心模块
│   │   ├── README.md                  # 语言包说明
│   │   ├── README-en.md               # 英文版本
│   │   ├── zh-CN.json                 # 简体中文
│   │   ├── zh-TW.json                 # 繁体中文
│   │   ├── en-US.json                 # 英语
│   │   ├── ja-JP.json                 # 日语
│   │   ├── ko-KR.json                 # 韩语
│   │   ├── de-DE.json                 # 德语
│   │   ├── es-ES.json                 # 西班牙语
│   │   ├── fr-FA.json                 # 波斯语
│   │   ├── ru-RU.json                 # 俄语
│   │   └── vi-VN.json                 # 越南语
│   │
│   ├── router/                        # 路由配置
│   │   └── index.ts                   # 路由表定义
│   │
│   ├── stores/                        # Pinia 状态管理
│   │   ├── index.ts                   # Pinia 实例初始化
│   │   ├── serverStore.ts             # 服务器列表和运行状态
│   │   ├── consoleStore.ts            # 控制台日志（切换页面不丢失）
│   │   ├── uiStore.ts                 # 界面状态（侧栏折叠等）
│   │   ├── settingsStore.ts           # 应用设置状态
│   │   ├── i18nStore.ts               # 国际化状态
│   │   ├── updateStore.ts             # 更新检查状态
│   │   ├── contextMenuStore.ts        # 上下文菜单状态
│   │   └── pluginStore.ts             # 插件状态管理
│   │
│   ├── styles/                        # 全局样式
│   │   ├── components/                # 组件样式
│   │   │   ├── common/                # 通用组件样式
│   │   │   ├── layout/                # 布局组件样式
│   │   │   └── views/                 # 页面组件样式
│   │   ├── views/                     # 页面样式
│   │   ├── variables.css              # CSS 变量（颜色、间距、圆角、字体、阴影）
│   │   ├── reset.css                  # 浏览器样式重置
│   │   ├── typography.css             # 排版样式
│   │   ├── animations.css             # 动画关键帧
│   │   ├── glass.css                  # 毛玻璃效果样式
│   │   ├── initial.css                # 初始样式
│   │   ├── app.css                    # 应用全局样式
│   │   └── plugin-list.css            # 插件列表样式
│   │
│   ├── themes/                        # 主题系统
│   │   ├── index.ts                   # 主题入口
│   │   ├── default.ts                 # 默认主题
│   │   ├── midnight.ts                # 午夜主题
│   │   ├── ocean.ts                   # 海洋主题
│   │   ├── rose.ts                    # 玫瑰主题
│   │   ├── sunset.ts                  # 日落主题
│   │   └── README.md                  # 主题说明
│   │
│   ├── types/                         # 类型定义
│   │   ├── common.ts                  # 通用类型定义
│   │   ├── plugin.ts                  # 插件相关类型定义
│   │   ├── server.ts                  # 服务器相关类型定义
│   │   └── theme.ts                   # 主题相关类型定义
│   │
│   ├── utils/                         # 工具函数
│   │   ├── constants.ts               # 常量定义
│   │   ├── errorHandler.ts            # 错误处理工具
│   │   ├── quoteUtils.ts              # 引号处理工具
│   │   ├── serverUtils.ts             # 服务器工具函数
│   │   ├── statsUtils.ts              # 统计工具函数
│   │   ├── theme.ts                   # 主题工具
│   │   └── version.ts                 # 版本工具
│   │
│   ├── views/                         # 页面视图（每个路由对应一个）
│   │   ├── HomeView.vue               # 首页（服务器列表、系统状态）
│   │   ├── CreateServerView.vue       # 创建/导入服务器页面
│   │   ├── ConsoleView.vue            # 控制台页面（实时日志、命令输入）
│   │   ├── ConfigView.vue             # 配置编辑页面（server.properties）
│   │   ├── PlayerView.vue             # 玩家管理页面（白名单、封禁、OP）
│   │   ├── SettingsView.vue           # 应用设置页面
│   │   ├── PaintView.vue              # 个性化设置页面
│   │   ├── AboutView.vue              # 关于页面（贡献者墙、更新检查）
│   │   ├── MarketView.vue             # 市场页面
│   │   ├── PluginsView.vue           # 插件列表页面
│   │   ├── PluginsPageView.vue       # 插件分页视图
│   │   ├── PluginCategoryView.vue     # 插件分类视图
│   │   ├── PluginPageView.vue         # 插件详情页面
│   │   └── DownloadFileView.vue       # 文件下载页面
│   │
│   ├── App.vue                        # 根组件
│   ├── main.ts                        # 应用入口（初始化 Vue、Pinia、Router）
│   ├── style.css                      # 样式汇总导入
│   └── vite-env.d.ts                  # Vite 环境类型声明
│
├── src-tauri/                         # 后端代码（Rust + Tauri 2）
│   │
│   ├── capabilities/                  # Tauri 权限配置
│   │   └── default.json               # 默认权限设置
│   │
│   ├── icons/                         # 应用图标
│   │   ├── 32x32.png                  # 32x32 图标
│   │   ├── 64x64.png                  # 64x64 图标
│   │   ├── 128x128.png                # 128x128 图标
│   │   ├── 128x128@2x.png             # 128x128 2x 图标
│   │   ├── icon.icns                  # macOS 图标
│   │   ├── icon.ico                   # Windows 图标
│   │   ├── icon.png                   # 通用图标
│   │   ├── source.png                 # 源图标
│   │   ├── Square30x30Logo.png        # 30x30 方形图标
│   │   ├── Square44x44Logo.png        # 44x44 方形图标
│   │   ├── Square71x71Logo.png        # 71x71 方形图标
│   │   ├── Square89x89Logo.png        # 89x89 方形图标
│   │   ├── Square107x107Logo.png      # 107x107 方形图标
│   │   ├── Square142x142Logo.png      # 142x142 方形图标
│   │   ├── Square150x150Logo.png      # 150x150 方形图标
│   │   ├── Square284x284Logo.png      # 284x284 方形图标
│   │   ├── Square310x310Logo.png      # 310x310 方形图标
│   │   ├── StoreLogo.png              # 商店图标
│   │   ├── android/                   # Android 图标
│   │   └── ios/                       # iOS 图标
│   │
│   ├── src/                           # Rust 源代码
│   │   ├── commands/                  # Tauri 命令（前端 invoke 调用的 API）
│   │   │   ├── mod.rs                 # 模块导出
│   │   │   ├── server.rs              # 服务器管理命令
│   │   │   ├── java.rs                # Java 检测命令
│   │   │   ├── config.rs              # 配置文件读写命令
│   │   │   ├── player.rs              # 玩家管理命令
│   │   │   ├── settings.rs            # 应用设置命令
│   │   │   ├── system.rs              # 系统信息、文件对话框命令
│   │   │   ├── update.rs              # 软件更新检查命令
│   │   │   ├── update_arch.rs         # 更新架构检测
│   │   │   ├── update_checksum.rs     # 更新校验和
│   │   │   ├── update_download.rs     # 更新下载
│   │   │   ├── update_github.rs       # GitHub 更新源
│   │   │   ├── update_install.rs      # 更新安装
│   │   │   ├── update_types.rs        # 更新类型定义
│   │   │   ├── update_version.rs      # 更新版本比较
│   │   │   ├── join.rs                # 加入服务器命令
│   │   │   ├── mods.rs                # 模组管理命令
│   │   │   ├── server_id.rs           # 服务器 ID 管理命令
│   │   │   ├── mcs_plugin.rs          # MCS 插件命令
│   │   │   ├── plugin.rs              # 插件系统命令
│   │   │   └── downloader.rs          # 下载管理命令
│   │   │
│   │   ├── services/                  # 业务逻辑层
│   │   │   ├── mod.rs                 # 模块导出
│   │   │   ├── server_manager.rs      # 服务器进程管理、日志读取
│   │   │   ├── java_detector.rs       # Java 环境扫描器
│   │   │   ├── java_installer.rs      # Java 安装器
│   │   │   ├── config_parser.rs       # .properties 文件解析器
│   │   │   ├── player_manager.rs      # 玩家数据文件读取
│   │   │   ├── settings_manager.rs    # 应用设置持久化
│   │   │   ├── mod_manager.rs         # 模组管理器
│   │   │   ├── join_manager.rs        # 加入服务器管理器
│   │   │   ├── server_id_manager.rs   # 服务器 ID 管理器
│   │   │   ├── mcs_plugin_manager.rs  # MCS 插件管理器
│   │   │   ├── async_loader.rs        # 异步加载器
│   │   │   ├── i18n.rs                # 国际化服务
│   │   │   ├── global.rs              # 全局单例管理器
│   │   │   ├── download_manager.rs    # 下载管理器
│   │   │   └── server_installer.rs    # 服务器安装器
│   │   │
│   │   ├── plugins/                   # 插件系统
│   │   │   ├── mod.rs                 # 插件模块导出
│   │   │   ├── api.rs                 # 插件 API
│   │   │   ├── loader.rs              # 插件加载器
│   │   │   ├── manager.rs             # 插件管理器
│   │   │   └── runtime/               # 插件运行时
│   │   │       ├── mod.rs             # 运行时模块导出
│   │   │       ├── api_bridge.rs      # API 桥接
│   │   │       ├── console.rs         # 控制台 API
│   │   │       ├── element.rs         # UI 元素 API
│   │   │       ├── filesystem.rs      # 文件系统 API
│   │   │       ├── helpers.rs         # 辅助函数
│   │   │       ├── http.rs            # HTTP API
│   │   │       ├── i18n.rs            # 国际化 API
│   │   │       ├── log.rs             # 日志 API
│   │   │       ├── plugins_api.rs     # 插件管理 API
│   │   │       ├── process.rs         # 进程 API
│   │   │       ├── server.rs          # 服务器 API
│   │   │       ├── storage.rs         # 存储 API
│   │   │       ├── system.rs          # 系统 API
│   │   │       └── ui.rs              # UI API
│   │   │
│   │   ├── models/                    # 数据结构定义
│   │   │   ├── mod.rs                 # 模块导出
│   │   │   ├── server.rs              # 服务器实例、状态数据结构
│   │   │   ├── config.rs              # 配置项数据结构
│   │   │   ├── settings.rs            # 应用设置数据结构
│   │   │   ├── mcs_plugin.rs          # MCS 插件数据结构
│   │   │   ├── plugin.rs              # 插件数据结构
│   │   │   └── download.rs            # 下载相关数据结构
│   │   │
│   │   ├── utils/                     # 工具函数
│   │   │   ├── mod.rs                 # 工具模块
│   │   │   ├── cli.rs                 # 命令行工具
│   │   │   ├── downloader.rs          # 下载工具
│   │   │   └── path.rs                # 路径工具
│   │   │
│   │   ├── lib.rs                     # Tauri 库入口（插件注册、命令注册）
│   │   └── main.rs                    # 应用主函数
│   │
│   ├── .gitignore                     # Git 忽略文件
│   ├── Cargo.lock                     # Rust 依赖锁定文件
│   ├── Cargo.toml                     # Rust 依赖配置
│   ├── build.rs                       # 构建脚本
│   └── tauri.conf.json                # Tauri 配置（窗口大小、标题、版本等）
│
│
├── docs/                              # 文档
│   ├── AI_GUIDE.md                    # AI 使用指南
│   ├── CONTRIBUTING.md                # 贡献指南
│   ├── STRUCTURE.md                   # 项目结构文档
│   └── 新手使用教程.html              # 新手使用教程
│
├── .github/                           # GitHub 配置
│   ├── ISSUE_TEMPLATE/                #  issue 模板
│   ├── workflows/                     # GitHub Actions 工作流
│   └── pull_request_template.md       # PR 模板
│
├── .vscode/                           # VS Code 配置
│   ├── extensions.json                # 推荐扩展
│   └── settings.json                  # 编辑器设置
│
├── .zed/                              # Zed 编辑器配置
│   └── settings.json                  # 编辑器设置
│
├── .SRCINFO                           # 包信息
├── .editorconfig                      # 编辑器配置
├── .gitattributes                     # Git 属性配置
├── .gitignore                         # Git 忽略文件
├── .oxfmtrc.json                      # Oxlint 格式化配置
├── .oxlintrc.json                     # Oxlint 配置
├── Cargo.lock                         # Rust 依赖锁定文件
├── Cargo.toml                         # Rust 依赖配置
├── LICENSE                            # 许可证文件
├── PKGBUILD                           # Arch Linux 包构建文件
├── README.md                          # 项目说明文档
├── README-en.md                       # 英文项目说明文档
├── index.html                         # HTML 入口文件
├── package-lock.json                  # Node.js 依赖锁定文件
├── package.json                       # Node.js 依赖配置
├── pnpm-lock.yaml                     # pnpm 依赖锁定文件
├── rustfmt.toml                       # Rust 代码格式化配置
├── sealantern.desktop                 # Linux 桌面文件
├── sealantern.install                 # 安装脚本
├── tsconfig.json                      # TypeScript 配置
├── tsconfig.node.json                 # Node.js 环境 TypeScript 配置
├── vite.config.ts                     # Vite 构建配置
└── 提交前测试必读！！！.md            # 提交前测试指南
```
