# Sea Lantern 国际化 (i18n) 指南

本文档介绍如何在 Sea Lantern 项目中使用和扩展国际化支持。

## 国际化架构

Sea Lantern 使用了自定义的国际化实现，基于以下核心组件：

- **翻译文件**：位于 `src/locales/` 目录下的 JSON 文件
- **i18n 核心**：`src/locales/index.ts` 中的 `I18n` 类
- **状态管理**：`src/stores/i18nStore.ts` 中的 Pinia store
- **语言切换**：`src/components/layout/AppHeader.vue` 中的语言选择菜单

## 支持的语言

目前项目支持以下语言：

- **简体中文** (`zh-CN`)：`src/locales/zh-CN.json`
- **繁体中文** (`zh-TW`)：`src/locales/zh-TW.json`
- **英文** (`en-US`)：`src/locales/en-US.json`

## 翻译文件结构

每个翻译文件都是一个 JSON 对象，按照功能模块组织键值对：

```json
{
  "common": {
    "app_name": "Sea Lantern",
    "home": "首页",
    "create_server": "创建服务器",
    "console": "控制台",
    "config_edit": "配置编辑",
    "player_manage": "玩家管理",
    "settings": "设置",
    "about": "关于",
    "loading": "加载中..."
  },
  "sidebar": {
    "groups": {
      "main": "通用",
      "server": "服务器",
      "system": "系统"
    },
    "collapse_btn": "收起侧栏"
  },
  "header": {
    "language": "语言",
    "chinese": "简中",
    "english": "EN",
    "chinese_tw": "繁中"
  },
  "home": {
    "title": "服务器管理",
    "no_servers": "暂无服务器",
    "create_first": "创建你的第一个服务器",
    "start": "启动",
    "stop": "停止",
    "edit": "编辑",
    "delete": "删除",
    "status": "状态",
    "running": "运行中",
    "stopped": "已停止",
    "starting": "启动中",
    "stopping": "停止中",
    "error": "异常",
    "system_resources": "系统资源",
    "detail_view": "切换到详细视图",
    "gauge_view": "切换到仪表盘",
    "recent_alerts": "最近警告 / 错误"
  }
  // 其他模块...
}
```

## 在组件中使用翻译

在 Vue 组件中，你可以使用 `i18n.t()` 方法来获取翻译文本：

### 基本用法

```vue
<script setup lang="ts">
import ...
...
import { i18n } from '../locales';
...
</script>
...
<template>
  <h1>{{ i18n.t("home.title") }}</h1>
  <button>{{ i18n.t("common.create_server") }}</button>
</template>
```

### 使用变量替换

对于包含动态内容的翻译，你可以使用变量替换：

```vue
<template>
  <div>{{ i18n.t("create.java_found", { count: javaList.length }) }}</div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { i18n } from "../locales";

const javaList = ref([
  /* Java 列表 */
]);
</script>
```

对应的翻译文件中：

```json
{
  "create": {
    "java_found": "找到 {{count}} 个 Java"
  }
}
```

## 添加新的翻译

当你添加新功能或修改现有功能时，需要更新所有语言的翻译文件：

1. **确定翻译键**：选择一个合适的模块和键名
2. **更新所有翻译文件**：在 `zh-CN.json`、`zh-TW.json` 和 `en-US.json` 中添加对应的翻译
3. **在组件中使用**：使用 `i18n.t()` 方法获取翻译文本

### 示例：添加新功能的翻译

假设你添加了一个新的 "备份" 功能：

1. 在 `src/locales/zh-CN.json` 中添加：

   ```json
   {
     "backup": {
       "title": "备份管理",
       "create_backup": "创建备份",
       "restore_backup": "恢复备份",
       "delete_backup": "删除备份"
     }
   }
   ```

2. 在 `src/locales/zh-TW.json` 中添加：

   ```json
   {
     "backup": {
       "title": "備份管理",
       "create_backup": "建立備份",
       "restore_backup": "恢復備份",
       "delete_backup": "刪除備份"
     }
   }
   ```

3. 在 `src/locales/en-US.json` 中添加：

   ```json
   {
     "backup": {
       "title": "Backup Management",
       "create_backup": "Create Backup",
       "restore_backup": "Restore Backup",
       "delete_backup": "Delete Backup"
     }
   }
   ```

4. 在组件中使用：
   ```vue
   <template>
     <h2>{{ i18n.t("backup.title") }}</h2>
     <button>{{ i18n.t("backup.create_backup") }}</button>
   </template>
   ```

## 添加新语言

要添加一种新的语言支持，需要以下步骤：

1. **创建翻译文件**：在 `src/locales/` 目录下创建新的 JSON 文件，例如 `ja-JP.json` 用于日语
2. **导入翻译文件**：在 `src/locales/index.ts` 中导入新的翻译文件并添加到 `translations` 对象中
3. **更新语言切换菜单**：在 `src/components/layout/AppHeader.vue` 中添加新的语言选项
4. **更新状态管理**：在 `src/stores/i18nStore.ts` 中更新相关逻辑
   - 如果需要为新语言添加特定的 getter（例如 `isJapanese`），在 getters 部分添加
   - 如果需要在 `toggleLocale` 方法中支持新语言的切换，修改该方法
5. **添加语言标签**：在所有翻译文件的 `header` 部分添加新语言的标签

### 示例：添加日语支持

1. 创建 `src/locales/ja-JP.json` 文件
2. 在 `src/locales/index.ts` 中添加：

   ```typescript
   import jaJP from "./ja-JP.json";

   const translations: Translations = {
     "zh-CN": zhCN,
     "en-US": enUS,
     "zh-TW": zhTW,
     "ja-JP": jaJP,
   };
   ```

3. 在 `src/components/layout/AppHeader.vue` 中添加：

   ```vue
   <div class="language-item" @click.stop="setLanguage('ja-JP')">
     {{ i18n.t('header.japanese') }}
   </div>
   ```

4. 在 `src/stores/i18nStore.ts` 中更新：
   - 在 getters 部分添加日语检测：
     ```typescript
     isJapanese: (state) => state.locale === "ja-JP",
     ```
   - 在 `toggleLocale` 方法中添加日语支持：
     ```typescript
     toggleLocale() {
       let newLocale = "zh-CN";
       if (this.locale === "zh-CN") {
         newLocale = "zh-TW";
       } else if (this.locale === "zh-TW") {
         newLocale = "en-US";
       } else if (this.locale === "en-US") {
         newLocale = "ja-JP";
       }
       this.setLocale(newLocale);
     },
     ```

5. 在所有翻译文件的 `header` 部分添加：
   ```json
   {
     "header": {
       "japanese": "日本語"
     }
   }
   ```

## 最佳实践

### 翻译键命名规范

- 使用小写字母和下划线
- 按照模块组织键名
- 保持键名简洁明了
- 避免使用硬编码的键名，使用常量或统一的命名约定

### 翻译文件管理

- 确保所有语言的翻译文件结构一致
- 当修改功能时，同步更新所有语言的翻译
- 对于较长的文本，可以考虑使用多行字符串或拆分为多个键
- 使用变量替换来处理动态内容，避免在翻译中硬编码具体值

## 故障排除

### 翻译不显示

1. **检查翻译键**：确保使用了正确的翻译键
2. **检查翻译文件**：确保所有翻译文件中都有对应的翻译
3. **检查语言设置**：确保当前选择的语言正确
4. **检查 i18n 实现**：确保 `i18n.t()` 方法的调用方式正确

### 变量不替换

1. **检查变量格式**：确保使用了 `{{variable}}` 格式
2. **检查参数传递**：确保传递了正确的选项对象
3. **检查变量名**：确保变量名与翻译文件中的一致

### 语言切换不生效

1. **检查语言代码**：确保使用了正确的语言代码（如 `zh-CN`）
2. **检查状态管理**：确保 `i18nStore.setLocale()` 方法被正确调用
3. **检查组件更新**：确保组件能够响应语言变化

## 总结

国际化是 Sea Lantern 项目的重要组成部分，它使得应用能够被来自不同地区的用户使用。通过遵循本指南，你可以确保你的贡献能够被所有语言的用户理解和使用。

如果你有任何关于国际化的问题或建议，欢迎在项目的 Issues 中提出。
