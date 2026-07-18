// 从 GitHub 获取原始 Markdown 并清理 Vue 语法

const GITHUB_RAW = "https://raw.githubusercontent.com/SeaLantern-Studio/sea-lantern-docs/main";
const DOCS_BASE = "https://docs.ideaflash.cn";

// Vue 模板变量（对应 vitepress/version.ts）
const vueVars: Record<string, string> = {
  VERSION: "1.3.0",
  ASSET_VERSION: "1.3.0",
  RPM_ASSET_VERSION: "1.3.0-1",
  ARCH_PKG_ASSET_VERSION: "1.3.0-1",
  RELEASE_BASE: "https://cnb.cool/SeaLantern-studio/SeaLantern/-/releases/download/v1.3.0",
};

/**
 * 清理 VitePress Markdown 中的 Vue 语法。
 */
function cleanVueSyntax(md: string, key: string): string {
  let result = md;

  // 1. 移除 <script setup>...</script> 块
  result = result.replace(/<script\s+setup[\s\S]*?<\/script>\s*/g, "");

  // 2. 移除自定义 Vue 组件标签
  result = result.replace(/<[A-Z][\w]*(?:\s+[\w-]+(?:=["'][\s\S]*?["'])?)*\s*\/?>\s*/g, "");

  // 3. Vue 模板表达式 {{ var }}
  result = result.replace(/\{\{\s*\w+\s*\}\}/g, "");

  // 4. 模板字面量变量替换
  for (const [name, value] of Object.entries(vueVars)) {
    result = result.replace(new RegExp("\\$\\{" + name + "\\}", "g"), value);
  }
  result = result.replace(/="`([^`]*)`"/g, '="$1"');
  result = result.replace(/:href/g, "href");

  // 5. 相对链接 → 绝对链接
  result = result.replace(/\]\(\/zh\/([^)]+)\)/g, "](" + DOCS_BASE + "/zh/$1)");

  // 6. 页面特定内容补充
  if (key === "intro") {
    result = result.replace(
      /## 特性[\s\S]*?(?=\n## )/,
      "## 特性\n\n" +
        getIntroFeatures() +
        "\n\n## 特性导读\n> 以上是 Sea Lantern 的主要功能概览，更多详细信息请查看各功能页面。",
    );
  }

  if (key === "contributor") {
    result +=
      "\n\n详见 [GitHub Contributors](https://github.com/SeaLantern-Studio/SeaLantern/graphs/contributors)\n";
  }

  return result.trim().replace(/\n{4,}/g, "\n\n\n");
}

function getIntroFeatures(): string {
  return [
    "### 实时控制台",
    "实时查看服务器日志，直接输入命令，支持命令历史。适合排查启动报错和日常运维，不必频繁切换外部终端。",
    "",
    "### 图形化配置",
    "server.properties 可视化编辑，按分类组织，减少手改文件。",
    "",
    "### 玩家管理",
    "白名单、封禁、OP 一键操作，常用管理集中在一个入口。",
    "",
    "### 插件系统",
    "基于 Lua 脚本扩展，支持自定义 UI 组件、右键菜单与插件市场。",
    "",
    "### 创建流程 2.0",
    "支持 JAR/脚本/已有服务器导入，智能识别开服方式并可自定义命令。",
    "",
    "### 主题系统",
    "内置 5 套主题，支持明暗模式；Windows 支持亚克力效果。",
    "",
    "### 多语言支持",
    "内置 10 种语言，覆盖中文、英语、日语、韩语等。支持运行时切换。",
    "",
    "### Java 管理",
    "自动检测已安装 Java，并支持一键下载安装。",
    "",
    "### Mod 管理",
    "查看服务端已安装的 Mod/插件，并提供基础管理能力。",
    "",
    "### 安全退出",
    "关闭软件时自动停止服务器，降低存档损坏风险。",
    "",
    "### 自动更新",
    "检查新版本并跳转下载页面（Arch Linux 使用 AUR 更新）。",
    "",
    "### 跨平台",
    "支持 Windows、macOS、Linux（含 Arch Linux AUR）。",
  ].join("\n");
}

/** 从 GitHub raw 获取 markdown 并清理 Vue 语法 */
export async function fetchDocContent(key: string): Promise<string> {
  const res = await fetch(GITHUB_RAW + "/zh/" + key + ".md");
  if (!res.ok) throw new Error("HTTP " + res.status);
  const md = await res.text();
  return cleanVueSyntax(md, key);
}

/** 文档站链接 */
export function getDocUrl(key: string): string {
  return DOCS_BASE + "/zh/" + key;
}
