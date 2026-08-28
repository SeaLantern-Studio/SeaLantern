# 前端设计契约

本文只保留会影响跨模块协作的前端契约。组件属性和后端方法的完整快照应从实际依赖或源码读取，不在仓库中长期复制一份容易失效的清单。

## 目录与传输

- `src/views` 和 `src/components` 负责页面与交互。
- `src/api` 是业务调用入口；统一导出位于 `src/api/index.ts`。
- `src/api/invoke.ts` 使用业务命令名在 Tauri 与 Axum 之间分发，Axum 映射只包含后端已经实现的路由。
- `src/api/tauri.ts` 提供原生 `invoke` 和环境判断。浏览器环境不能调用未迁移的 Tauri command；调用方应处理 `NotImplementedError` 或使用 `silent` 默认值。
- 新的共享业务先落到 API 模块，不在页面组件中直接拼装 HTTP 路径。宿主专用 UI 可以使用对应宿主 API，但不能让页面承担后端业务规则。

## 国际化

实现位于 `src/language`：Vite 通过 `import.meta.glob("./*.json", { eager: true })` 加载 JSON 语言包，默认 locale 是 `zh-CN`，缺失文本回退到 `en-US`。翻译键使用点分路径，插值同时支持 `{{name}}` 和 `{name}`。

插件语言通过 `registerPluginLocale`、`addPluginTranslations` 和 `removePluginTranslations` 注入；插件翻译只补充应用翻译，不应覆盖已有应用键。修改语言文件时保留所有插值键，并检查所有受影响页面。

## 主题

实现位于 `src/themes`，类型位于 `src/types/theme.ts`。主题文件使用 Vite glob 自动发现，主题定义需要 `light`、`dark`、`lightAcrylic` 和 `darkAcrylic` 四种 `ColorPlan`。注册、注销、查询和重置入口由 `src/themes/index.ts` 提供；`mapLegacyPlanName` 只用于现存旧配置映射。

新增主题只应添加完整的 `ThemeDefinition` 并验证文字对比度，不要在页面样式中另造一套主题状态或颜色计划。

## UI 依赖

项目使用 `cmzya-modern-ui`，版本和入口以根目录 `package.json` 与 `src/main.ts` 为准。应用入口显式导入 `cmzya-modern-ui/style.css`，并集中注册公共组件；应用自己的颜色与页面样式位于 `src/style.css` 和 `src/styles`。

UI 组件 API 发生变化时，先检查安装包的类型声明和实际导出，再更新调用方。不要把旧的 `docs/ui库.md` 全量复制回来；只在出现稳定的项目级使用约束时补充本文。

## 源码锚点

- [API exports](../../src/api/index.ts)
- [dual transport adapter](../../src/api/invoke.ts)
- [i18n implementation](../../src/language/index.ts)
- [theme implementation](../../src/themes/index.ts)
- [theme types](../../src/types/theme.ts)
- [frontend entry](../../src/main.ts)
