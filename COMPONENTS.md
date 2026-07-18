# CmzYa Modern UI — 组件文档

## 概述

CmzYa Modern UI 是一个轻量级 Vue 3 组件库，共 **18 个组件**，所有样式通过 CSS 变量驱动，支持深浅主题切换。

### 安装

```bash
npm install cmzya-modern-ui
# 或
pnpm add cmzya-modern-ui
```

### 全局注册

```ts
import { createApp } from "vue";
import App from "./App.vue";
import "cmzya-modern-ui/dist/style.css";
import * as CmzYa from "cmzya-modern-ui";

const app = createApp(App);
// 全量注册（使用时用 kebab-case）
for (const [name, comp] of Object.entries(CmzYa)) {
  if (name.startsWith("Cmz_")) {
    app.component(name.replace(/_/g, "-"), comp);
  }
}
app.mount("#app");
```

### 按需引入

```vue
<script setup>
import { Cmz_Button, Cmz_Toggle } from "cmzya-modern-ui";
import "cmzya-modern-ui/dist/style.css";
</script>
```

### 别名映射

| 别名                  | 实际组件   |
| --------------------- | ---------- |
| `Cmz_Checkbox`        | Cmz_Toggle |
| `Cmz_Switch`          | Cmz_Toggle |
| `Cmz_Textarea`        | Cmz_Input  |
| `Cmz_StatusIndicator` | Cmz_Badge  |

---

## 1. Cmz_Button 按钮

### Props

| Prop     | 类型                              | 默认值          | 说明                            |
| -------- | --------------------------------- | --------------- | ------------------------------- |
| variant  | `"solid" \| "outline" \| "ghost"` | `"solid"`       | 变体                            |
| size     | `"sm" \| "md" \| "lg"`            | `"md"`          | 尺寸                            |
| color    | `string`                          | `--cmz-primary` | 自定义颜色（传任意 CSS 颜色值） |
| type     | `"button" \| "submit" \| "reset"` | `"button"`      | 原生 type                       |
| disabled | `boolean`                         | `false`         | 禁用                            |
| loading  | `boolean`                         | `false`         | 加载中（显示旋转图标）          |
| iconOnly | `boolean`                         | `false`         | 纯图标模式（去除内边距）        |

### Slots

| Slot    | 说明            |
| ------- | --------------- |
| default | 按钮文字 / 图标 |

### 示例

```vue
<cmz-button>默认</cmz-button>
<cmz-button variant="outline">描边</cmz-button>
<cmz-button variant="ghost">幽灵</cmz-button>
<cmz-button size="sm">小号</cmz-button>
<cmz-button size="lg">大号</cmz-button>
<cmz-button color="#22c55e">自定义色</cmz-button>
<cmz-button loading>加载中</cmz-button>
<cmz-button disabled>禁用</cmz-button>
<cmz-button iconOnly>
  <svg><!-- 图标 --></svg>
</cmz-button>
```

---

## 2. Cmz_Input / Cmz_Textarea 输入框

### Props

| Prop               | 类型                                             | 默认值       | 说明                 |
| ------------------ | ------------------------------------------------ | ------------ | -------------------- |
| modelValue         | `string`                                         | `""`         | v-model 绑定值       |
| placeholder        | `string`                                         | `""`         | 占位文字             |
| label              | `string`                                         | `""`         | 标签文字             |
| disabled           | `boolean`                                        | `false`      | 禁用                 |
| maxlength          | `number`                                         | —            | 最大字符数           |
| multiline          | `boolean`                                        | `false`      | 是否多行（textarea） |
| rows               | `number`                                         | `4`          | 多行行数             |
| resize             | `"none" \| "vertical" \| "horizontal" \| "both"` | `"vertical"` | 多行调整大小         |
| type               | `string`                                         | `"text"`     | 输入类型             |
| min                | `number`                                         | —            | 数字最小值           |
| max                | `number`                                         | —            | 数字最大值           |
| step               | `number`                                         | —            | 数字步进             |
| hideNumberControls | `boolean`                                        | `true`       | 隐藏数字上下箭头     |

### Events

| Event             | 参数     | 说明           |
| ----------------- | -------- | -------------- |
| update:modelValue | `string` | 输入变化时触发 |

### 示例

```vue
<cmz-input v-model="text" placeholder="请输入" />
<cmz-input v-model="text" label="用户名" />
<cmz-input v-model="text" maxlength="20" />
<cmz-input v-model="number" type="number" :min="0" :max="100" />

<!-- 使用别名：多行文本 -->
<cmz-textarea v-model="desc" :rows="6" placeholder="请输入描述" />
<cmz-textarea v-model="desc" resize="both" />
```

---

## 3. Cmz_Field 复合输入域

整合 text / number / select 三种输入模式。

### Props

| Prop        | 类型                             | 默认值   | 说明            |
| ----------- | -------------------------------- | -------- | --------------- |
| modelValue  | `string \| number`               | `""`     | v-model 绑定值  |
| variant     | `"text" \| "number" \| "select"` | `"text"` | 变体            |
| placeholder | `string`                         | `""`     | 占位文字        |
| label       | `string`                         | `""`     | 标签文字        |
| disabled    | `boolean`                        | `false`  | 禁用            |
| options     | `SelectOption[]`                 | —        | select 选项列表 |
| min         | `number`                         | —        | number 最小值   |
| max         | `number`                         | —        | number 最大值   |
| step        | `number`                         | `1`      | number 步进     |

### SelectOption

```ts
interface SelectOption {
  label: string;
  value: string | number;
}
```

### 示例

```vue
<cmz-field v-model="text" variant="text" placeholder="文本" />
<cmz-field v-model="num" variant="number" :min="0" :max="100" />
<cmz-field
  v-model="sel"
  variant="select"
  :options="[
    { label: '选项A', value: 'a' },
    { label: '选项B', value: 'b' },
  ]"
/>
<cmz-field v-model="text" label="姓名" />
```

---

## 4. Cmz_Select 下拉选择

### Props

| Prop          | 类型                | 默认值     | 说明             |
| ------------- | ------------------- | ---------- | ---------------- |
| modelValue    | `string \| number`  | —          | v-model 绑定值   |
| options       | `SelectOption[]`    | **必填**   | 选项列表         |
| label         | `string`            | —          | 标签文字         |
| placeholder   | `string`            | `"Select"` | 占位文字         |
| disabled      | `boolean`           | `false`    | 禁用             |
| searchable    | `boolean`           | `false`    | 可搜索           |
| loading       | `boolean`           | `false`    | 加载状态         |
| maxHeight     | `string`            | `"280px"`  | 下拉面板最大高度 |
| collapsed     | `boolean`           | `false`    | 紧凑模式         |
| dropdownAlign | `"left" \| "right"` | `"left"`   | 下拉对齐方向     |
| dropdownWidth | `string`            | `"200px"`  | 下拉面板宽度     |
| icon          | `Component`         | —          | 自定义图标组件   |

### SelectOption

```ts
interface SelectOption {
  label: string;
  value: string | number;
  subLabel?: string; // 副标题
}
```

### 示例

```vue
<cmz-select v-model="val" :options="options" />
<cmz-select v-model="val" :options="options" searchable placeholder="搜索..." />
<cmz-select v-model="val" :options="options" loading />
<cmz-select v-model="val" :options="options" dropdownAlign="right" />
<cmz-select v-model="val" :options="options" collapsed />
```

---

## 5. Cmz_Toggle / Cmz_Checkbox / Cmz_Switch 开关

### Props

| Prop          | 类型                     | 默认值     | 说明                    |
| ------------- | ------------------------ | ---------- | ----------------------- |
| modelValue    | `boolean`                | `false`    | v-model 绑定值          |
| variant       | `"checkbox" \| "switch"` | `"switch"` | 变体                    |
| size          | `"sm" \| "md" \| "lg"`   | `"md"`     | 尺寸                    |
| label         | `string`                 | —          | 标签文字                |
| disabled      | `boolean`                | `false`    | 禁用                    |
| indeterminate | `boolean`                | `false`    | 半选状态（仅 checkbox） |
| readonly      | `boolean`                | `false`    | 只读                    |

### Events

| Event             | 参数      | 说明     |
| ----------------- | --------- | -------- |
| update:modelValue | `boolean` | v-model  |
| change            | `boolean` | 点击变化 |

### 示例

```vue
<cmz-toggle v-model="val">开关</cmz-toggle>
<cmz-toggle v-model="val" size="sm" />
<cmz-toggle v-model="val" size="lg" />
<!-- 别名 -->
<cmz-switch v-model="val" label="WiFi" />
<cmz-checkbox v-model="checked" label="同意条款" />
<cmz-checkbox v-model="checked" indeterminate />
```

---

## 6. Cmz_Badge / Cmz_StatusIndicator 徽章

### Props

| Prop     | 类型                             | 默认值     | 说明         |
| -------- | -------------------------------- | ---------- | ------------ |
| text     | `string`                         | —          | 显示文字     |
| color    | `string`                         | —          | 自定义颜色   |
| size     | `"small" \| "medium" \| "large"` | `"medium"` | 尺寸         |
| closable | `boolean`                        | `false`    | 可关闭       |
| maxWidth | `number \| string`               | —          | 文字最大宽度 |
| dot      | `boolean`                        | `false`    | 圆点模式     |
| pulse    | `boolean`                        | `false`    | 圆点脉冲动画 |

### Events

| Event | 参数 | 说明         |
| ----- | ---- | ------------ |
| close | —    | 点击关闭按钮 |

### 变体组合

| 组合              | 效果                       |
| ----------------- | -------------------------- |
| `text="标签"`     | 纯文字胶囊                 |
| `dot`             | 纯圆点（胶囊缩小包裹圆点） |
| `dot text="在线"` | 圆点 + 文字组合胶囊        |
| `dot pulse`       | 圆点 + 脉冲动画            |

### 示例

```vue
<cmz-badge text="新" />
<cmz-badge text="标签" closable @close="..." />
<cmz-badge text="VIP" color="#f59e0b" />
<cmz-badge text="长文本标签" maxWidth="80" />
<cmz-badge size="small" text="小" />
<cmz-badge size="large" text="大" />
<cmz-badge dot />
<cmz-badge dot pulse />
<cmz-badge dot text="在线" />
<cmz-badge dot text="忙碌" color="#ef4444" pulse />
```

---

## 7. Cmz_Card 卡片

### Props

| Prop      | 类型                                            | 默认值    | 说明         |
| --------- | ----------------------------------------------- | --------- | ------------ |
| title     | `string`                                        | —         | 标题         |
| subtitle  | `string`                                        | —         | 副标题       |
| hoverable | `boolean`                                       | `false`   | 悬停提升效果 |
| padding   | `"none" \| "xs" \| "sm" \| "md" \| "lg"`        | `"md"`    | 内边距       |
| variant   | `"solid" \| "glass" \| "outline" \| "elevated"` | `"solid"` | 变体         |

### Slots

| Slot    | 说明                          |
| ------- | ----------------------------- |
| default | 卡片正文                      |
| header  | 标题区（替换 title/subtitle） |
| actions | 标题栏右侧操作区              |

### 示例

```vue
<cmz-card title="标题" subtitle="副标题">内容</cmz-card>
<cmz-card variant="glass">毛玻璃</cmz-card>
<cmz-card variant="outline">描边</cmz-card>
<cmz-card variant="elevated">浮动阴影</cmz-card>
<cmz-card hoverable>可悬停</cmz-card>
<cmz-card padding="lg">大内边距</cmz-card>
<cmz-card>
  <template #actions>
    <cmz-button size="sm">操作</cmz-button>
  </template>
  正文
</cmz-card>
```

---

## 8. Cmz_Modal 模态框

### Props

| Prop            | 类型      | 默认值    | 说明                         |
| --------------- | --------- | --------- | ---------------------------- |
| visible         | `boolean` | **必填**  | 显示/隐藏                    |
| title           | `string`  | —         | 标题                         |
| width           | `string`  | `"480px"` | 宽度                         |
| closeOnOverlay  | `boolean` | `true`    | 点击遮罩关闭                 |
| autoClose       | `number`  | `0`       | 自动关闭毫秒数（0=不自动关） |
| showCloseButton | `boolean` | `true`    | 显示关闭按钮                 |

### Events

| Event | 说明       |
| ----- | ---------- |
| close | 关闭时触发 |

### Slots

| Slot    | 说明   |
| ------- | ------ |
| default | 正文   |
| header  | 标题区 |

### 示例

```vue
<cmz-modal :visible="show" title="提示" @close="show = false">
  <p>确认删除？</p>
</cmz-modal>

<cmz-modal :visible="show" width="600px" :close-on-overlay="false">
  自定义头部
</cmz-modal>

<cmz-modal :visible="show" :auto-close="3000">
  3秒自动关闭
</cmz-modal>
```

---

## 9. Cmz_Toast 通知 + useToast 命令式 API

### 组件 Props

| Prop       | 类型            | 默认值        | 说明           |
| ---------- | --------------- | ------------- | -------------- |
| position   | `ToastPosition` | `"top-right"` | 显示位置       |
| maxVisible | `number`        | `5`           | 最多同时显示数 |

### ToastPosition

```ts
type ToastPosition =
  | "top-right"
  | "top-left"
  | "bottom-right"
  | "bottom-left"
  | "top-center"
  | "bottom-center";
```

### useToast 命令式 API

```ts
import { useToast } from "cmzya-modern-ui";

const toast = useToast({ max: 5 }); // 可选：设置队列容量

toast.success("操作成功");
toast.error({ title: "错误", description: "详细信息", duration: 5000 });
toast.warning("警告提示");
toast.info("这是一条信息");
toast.loading({ title: "上传中...", duration: 0 }); // duration=0 常驻

// 手动移除
const id = toast.success("保存成功");
toast.remove(id);

// 清空所有
toast.clear();

// 动态调整容量
toast.setMax(3);
```

### ToastOptions

```ts
interface ToastOptions {
  title?: string;
  description?: string;
  type?: "success" | "error" | "warning" | "info" | "loading";
  color?: string; // 自定义颜色
  duration?: number; // 毫秒，0=常驻，默认根据 type 不同
  closable?: boolean;
  icon?: Component; // 自定义图标
  action?: {
    label: string;
    onClick: () => void;
  };
}
```

### 使用方式

在根组件放一个 `<cmz-toast>`，任意页面调用 `useToast().success()`：

```vue
<!-- App.vue -->
<template>
  <cmz-toast position="top-right" />
  <router-view />
</template>
```

---

## 10. Cmz_Console 控制台

### Props

| Prop           | 类型            | 默认值    | 说明                   |
| -------------- | --------------- | --------- | ---------------------- |
| lines          | `ConsoleLine[]` | **必填**  | 日志行数据             |
| showTimestamps | `boolean`       | `false`   | 是否显示时间戳边框     |
| autoScroll     | `boolean`       | `true`    | 自动滚动到底部         |
| maxLines       | `number`        | `5000`    | 显示行数上限           |
| height         | `string`        | `"400px"` | 容器高度               |
| readonly       | `boolean`       | `false`   | 只读模式（隐藏输入框） |
| placeholder    | `string`        | `""`      | 无日志时的占位文字     |
| selectionColor | `string`        | `""`      | 自定义选择高亮色       |

### ConsoleLine

```ts
interface ConsoleLine {
  text: string;
  type?: "input" | "output" | "error" | "warning" | "info" | "success" | "system";
  timestamp?: string;
  special?: "rainbow"; // 彩虹文字
}
```

### 日志级别识别

控制台自动识别行内 `[...]` 括号中的日志级别并渲染为颜色胶囊：

| 级别词                                   | 胶囊颜色 |
| ---------------------------------------- | -------- |
| INFO                                     | 灰色     |
| SUCCESS                                  | 绿色     |
| ERROR / FATAL / CRITICAL / EMERG / ALERT | 红色     |
| WARN / WARNING                           | 琥珀     |
| DEBUG / TRACE / VERBOSE                  | 紫色     |
| NOTICE                                   | 青色     |

时间戳 `[HH:MM:SS]` 自动识别为主题色边框。

### 内置彩蛋命令

- `whoami` — 显示彩虹渐变的 "CmzYa"（组件内部处理，不触发 command 事件）

### Events

| Event   | 参数           | 说明           |
| ------- | -------------- | -------------- |
| command | `text: string` | 用户输入命令   |
| clear   | —              | 清屏（Ctrl+L） |

### 示例

```vue
<cmz-console :lines="logs" @command="handleCommand" :auto-scroll="true" />
```

```ts
// 日志数据
const logs = ref([
  { text: "[INFO] 系统启动", type: "output" as const },
  { text: "[SUCCESS] 连接成功", type: "output" as const },
  { text: "[WARN] 磁盘空间不足", type: "warning" as const },
  { text: "[ERROR] 连接超时", type: "error" as const },
  { text: "[10:23:47] [DEBUG] 请求耗时 120ms", type: "output" as const },
]);
```

---

## 11. Cmz_TabBar 标签栏

### Props

| Prop       | 类型           | 默认值   | 说明                       |
| ---------- | -------------- | -------- | -------------------------- |
| modelValue | `T`            | **必填** | 当前激活 key               |
| tabs       | `TabBarItem[]` | **必填** | 标签列表                   |
| level      | `1 \| 2`       | `1`      | 层级（1=风格一，2=风格二） |
| vertical   | `boolean`      | `false`  | 垂直排列                   |

### TabBarItem

```ts
interface TabBarItem<T = string | null> {
  key: T;
  label: string;
  count?: number | string; // 徽标数字
  countTitle?: string; // 徽标 title
  icon?: string;
  suffixIcon?: Component; // 右侧图标
  suffixTitle?: string; // 右侧图标 title
  disabled?: boolean;
}
```

### 示例

```vue
<cmz-tab-bar
  v-model="active"
  :tabs="[
    { key: 'tab1', label: '选项一' },
    { key: 'tab2', label: '选项二', count: 3 },
    { key: 'tab3', label: '选项三', disabled: true },
  ]"
/>

<cmz-tab-bar v-model="active" :tabs="tabs" level="2" />
<cmz-tab-bar v-model="active" :tabs="tabs" vertical />
```

---

## 12. Cmz_Divider 分割线

### Props

| Prop        | 类型                              | 默认值         | 说明               |
| ----------- | --------------------------------- | -------------- | ------------------ |
| orientation | `"horizontal" \| "vertical"`      | `"horizontal"` | 方向               |
| label       | `string`                          | —              | 带文字的水平分割线 |
| variant     | `"solid" \| "dashed" \| "dotted"` | `"solid"`      | 样式               |
| thickness   | `"thin" \| "normal" \| "thick"`   | `"normal"`     | 粗细               |

### 示例

```vue
<cmz-divider />
<cmz-divider label="或" />
<cmz-divider variant="dashed" />
<cmz-divider variant="dotted" />
<cmz-divider orientation="vertical" />
```

---

## 13. Cmz_Spinner 加载动画

### Props

| Prop | 类型                   | 默认值 | 说明 |
| ---- | ---------------------- | ------ | ---- |
| size | `"sm" \| "md" \| "lg"` | `"md"` | 尺寸 |

### 示例

```vue
<cmz-spinner />
<cmz-spinner size="sm" />
<cmz-spinner size="lg" />
```

---

## 14. Cmz_Progress 进度条

### Props

| Prop        | 类型      | 默认值          | 说明         |
| ----------- | --------- | --------------- | ------------ |
| value       | `number`  | **必填**        | 当前进度值   |
| max         | `number`  | `100`           | 最大值       |
| label       | `string`  | —               | 左侧标签文字 |
| showPercent | `boolean` | `true`          | 显示百分比   |
| color       | `string`  | `--cmz-primary` | 自定义颜色   |

### 示例

```vue
<cmz-progress :value="65" />
<cmz-progress :value="45" :max="50" />
<cmz-progress :value="80" label="下载进度" />
<cmz-progress :value="30" :show-percent="false" />
<cmz-progress :value="90" color="#22c55e" />
```

---

## 15. Cmz_Dropzone 文件拖放区

### Props

| Prop           | 类型       | 默认值                  | 说明                 |
| -------------- | ---------- | ----------------------- | -------------------- |
| modelValue     | `string`   | `""`                    | 绑定值               |
| label          | `string`   | `""`                    | 主标题文字           |
| subLabel       | `string`   | `""`                    | 副标题               |
| badge          | `string`   | `""`                    | 徽标文字             |
| disabled       | `boolean`  | `false`                 | 禁用                 |
| loading        | `boolean`  | `false`                 | 加载状态             |
| acceptFolders  | `boolean`  | `true`                  | 接受文件夹           |
| acceptFiles    | `boolean`  | `true`                  | 接受文件             |
| fileExtensions | `string[]` | `[".zip", ".tar", ...]` | 接受的文件扩展名     |
| placeholder    | `string`   | `""`                    | 占位文字             |
| clearable      | `boolean`  | `true`                  | 可清除               |
| multiple       | `boolean`  | `false`                 | 多选                 |
| isDragging     | `boolean`  | undefined               | 拖拽状态（外部控制） |

### Events

| Event             | 参数              | 说明           |
| ----------------- | ----------------- | -------------- |
| drop              | `path: string`    | 文件拖放完成   |
| dropMultiple      | `paths: string[]` | 多文件拖放完成 |
| clear             | —                 | 清除           |
| click             | —                 | 点击区域       |
| error             | `message: string` | 错误           |
| update:isDragging | `boolean`         | 拖拽状态变化   |

### 示例

```vue
<cmz-dropzone label="上传文件" subLabel="拖拽或点击上传" />
<cmz-dropzone :file-extensions="['.jpg', '.png']" />
<cmz-dropzone loading badge="处理中" />
<cmz-dropzone multiple clearable />
```

---

## 16. Cmz_Tooltip 提示框

### Props

| Prop      | 类型                                     | 默认值   | 说明             |
| --------- | ---------------------------------------- | -------- | ---------------- |
| content   | `string`                                 | **必填** | 提示文字         |
| delay     | `number`                                 | —        | 显示延迟（毫秒） |
| placement | `"top" \| "bottom" \| "left" \| "right"` | `"top"`  | 弹出方向         |

### 示例

```vue
<cmz-tooltip content="上方提示">悬停</cmz-tooltip>
<cmz-tooltip content="下方提示" placement="bottom">悬停</cmz-tooltip>
<cmz-tooltip content="左侧提示" placement="left">悬停</cmz-tooltip>
<cmz-tooltip content="右侧提示" placement="right">悬停</cmz-tooltip>
<cmz-tooltip content="延迟显示" :delay="500">悬停</cmz-tooltip>
```

---

## 17. Cmz_Markdown Markdown 渲染器

### Props

| Prop            | 类型                | 默认值    | 说明          |
| --------------- | ------------------- | --------- | ------------- |
| content         | `string`            | **必填**  | MD 原文       |
| roundedTable    | `boolean`           | `true`    | 表格圆角      |
| codeHighlight   | `boolean`           | `true`    | 代码语法高亮  |
| sanitized       | `boolean`           | `true`    | 过滤危险 HTML |
| variant         | `"plain" \| "card"` | `"plain"` | 容器样式      |
| features        | `MarkdownFeatures`  | 全开      | 特殊语法开关  |
| listLayout      | `"stack" \| "grid"` | `"stack"` | 列表布局      |
| listGridColumns | `number \| "auto"`  | `3`       | grid 列数     |

### MarkdownFeatures

```ts
interface MarkdownFeatures {
  alert?: boolean; // 警示框 !>
  linkCard?: boolean; // 链接卡片 =>
  container?: boolean; // 容器块 :::
}
```

### 特殊语法

**警示框 `!>`**

```markdown
!> [info] 这是一条信息提示
!> [warning] 注意警告
!> [error] 错误提示
!> [success] 操作成功
!> [tip] 小贴士
```

**链接卡片 `=>`**

```markdown
=> 文本描述 | https://example.com
```

**容器块 `:::`**

```markdown
::: tip
嵌套的 Markdown 内容...
:::
```

### 示例

```vue
<cmz-markdown :content="mdContent" />
<cmz-markdown :content="mdContent" variant="card" />
<cmz-markdown :content="mdContent" list-layout="grid" :list-grid-columns="3" />
<cmz-markdown :content="mdContent" list-layout="grid" list-grid-columns="auto" />
<cmz-markdown :content="mdContent" :code-highlight="false" />
<cmz-markdown :content="mdContent" :features="{ alert: true, linkCard: false }" />
```

---

## 18. Cmz_FormField 表单字段容器

### Props

| Prop          | 类型              | 默认值  | 说明        |
| ------------- | ----------------- | ------- | ----------- |
| label         | `string`          | —       | 标签文字    |
| required      | `boolean`         | `false` | 显示必填 \* |
| error         | `string`          | —       | 错误提示    |
| hint          | `string`          | —       | 提示文字    |
| labelPosition | `"top" \| "left"` | `"top"` | 标签位置    |

### Slots

| Slot    | 说明     |
| ------- | -------- |
| default | 表单控件 |

### 示例

```vue
<cmz-form-field label="用户名" required>
  <cmz-input placeholder="请输入" />
</cmz-form-field>

<cmz-form-field label="密码" error="密码不能为空">
  <cmz-input type="password" />
</cmz-form-field>

<cmz-form-field label="描述" hint="最多 200 字">
  <cmz-textarea />
</cmz-form-field>

<cmz-form-field label="设置" label-position="left">
  <cmz-toggle />
</cmz-form-field>
```

---

## CSS 变量体系

所有样式通过 CSS 变量驱动，修改 `variables.css` 即可全局换肤。

### 颜色

```css
--cmz-primary: #0ea5e9; /* 主色 */
--cmz-primary-light: #7dd3fc; /* 主色 - 浅 */
--cmz-primary-dark: #0369a1; /* 主色 - 深 */
--cmz-primary-bg: rgba(14, 165, 233, 0.08); /* 主色背景 */

--cmz-accent: #06b6d4; /* 强调色 */
--cmz-accent-light: #67e8f9;
```

### 状态色

```css
--cmz-success: #22c55e; /* 成功 - 绿色 */
--cmz-success-bg: rgba(34, 197, 94, 0.1);
--cmz-warning: #f59e0b; /* 警告 - 琥珀 */
--cmz-warning-bg: rgba(245, 158, 11, 0.1);
--cmz-error: #ef4444; /* 错误 - 红色 */
--cmz-error-bg: rgba(239, 68, 68, 0.1);
--cmz-info: #3b82f6; /* 信息 - 蓝色 */
--cmz-info-bg: rgba(59, 130, 246, 0.1);
--cmz-debug: #c084fc; /* 调试 - 紫色 */
--cmz-debug-bg: rgba(192, 132, 252, 0.12);
--cmz-notice: #22d3ee; /* 注意 - 青色 */
--cmz-notice-bg: rgba(34, 211, 238, 0.1);
```

### 三层背景体系

```css
--cmz-bg: #eef2f7; /* 一级：页面底色 */
--cmz-bg-secondary: #ffffff; /* 二级：卡片/模态框/容器 */
--cmz-bg-tertiary: #f5f7fa; /* 三级：输入框/标签栏等容器内元素 */
```

### 文字色

```css
--cmz-text-primary: #0f172a; /* 主文字 */
--cmz-text-secondary: #475569; /* 辅助文字 */
--cmz-text-tertiary: #64748b; /* 提示文字 */
--cmz-text-inverse: #ffffff; /* 反色文字 */
```

### 圆角

```css
--cmz-radius-xs: 4px;
--cmz-radius-sm: 6px;
--cmz-radius-md: 10px;
--cmz-radius-lg: 16px;
--cmz-radius-xl: 24px;
--cmz-radius-full: 9999px;
--cmz-radius-scrollbar: 4px;
```

### 间距

```css
--cmz-space-xs: 4px;
--cmz-space-sm: 8px;
--cmz-space-md: 16px;
--cmz-space-lg: 24px;
--cmz-space-xl: 32px;
--cmz-space-2xl: 48px;
```

### 字体

```css
--cmz-font-sans: "..."; /* 无衬线正文 */
--cmz-font-mono: "..."; /* 等宽（代码） */
--cmz-font-display: "..."; /* 展示 */
```

### 字号

```css
--cmz-font-size-xs: 0.75rem; /* 12px */
--cmz-font-size-sm: 0.8125rem; /* 13px */
--cmz-font-size-base: 0.875rem; /* 14px */
--cmz-font-size-lg: 1rem; /* 16px */
--cmz-font-size-xl: 1.125rem; /* 18px */
--cmz-font-size-2xl: 1.25rem; /* 20px */
--cmz-font-size-3xl: 1.5rem; /* 24px */
--cmz-font-size-4xl: 2rem; /* 32px */
```

### 阴影

```css
--cmz-shadow-sm;
--cmz-shadow-md;
--cmz-shadow-lg;
--cmz-shadow-xl;
--cmz-shadow-elevated;
--cmz-shadow-card;
--cmz-shadow-button;
--cmz-shadow-button-hover;
--cmz-shadow-input;
--cmz-shadow-input-focus;
```

### 毛玻璃 / 丙烯酸 (Acrylic)

毛玻璃背景色（供 `backdrop-filter` 配合使用，自动适应深浅主题）：

```css
/* 模糊半径 */
--cmz-blur-sm: 8px;
--cmz-blur-md: 16px;
--cmz-blur-lg: 24px;
--cmz-blur-xl: 32px;
--cmz-blur-xxl: 40px;
/* 饱和度增强 */
--cmz-saturate-subtle: 150%;
--cmz-saturate-normal: 180%;
--cmz-saturate-strong: 200%;
/* 毛玻璃背景色（卡片级，半透明 + blur 配合使用） */
--cmz-glass-bg: rgba(255, 255, 255, 0.65); /* 浅色 */
--cmz-glass-border: rgba(255, 255, 255, 0.45); /* 浅色 */
/* 丙烯酸（浮层级，更高透明 + 强模糊，适合弹出层/下拉菜单） */
--cmz-acrylic-bg: rgba(255, 255, 255, 0.55); /* 浅色 */
--cmz-acrylic-bg-strong: rgba(255, 255, 255, 0.72);
--cmz-acrylic-border: rgba(255, 255, 255, 0.35);
--cmz-acrylic-blur: var(--cmz-blur-lg); /* 默认 24px */
--cmz-acrylic-saturate: var(--cmz-saturate-normal); /* 默认 180% */
```

深色模式自动切换为：

```css
--cmz-glass-bg: rgba(30, 33, 48, 0.72);
--cmz-glass-border: rgba(255, 255, 255, 0.08);
--cmz-acrylic-bg: rgba(30, 33, 48, 0.62);
--cmz-acrylic-bg-strong: rgba(30, 33, 48, 0.78);
--cmz-acrylic-border: rgba(255, 255, 255, 0.06);
```

使用示例（组件内无需重复定义，直接引用变量即可）：

```css
/* 浮层毛玻璃 */
.element {
  background: var(--cmz-acrylic-bg);
  backdrop-filter: blur(var(--cmz-acrylic-blur)) saturate(var(--cmz-acrylic-saturate));
  -webkit-backdrop-filter: blur(var(--cmz-acrylic-blur)) saturate(var(--cmz-acrylic-saturate));
  border: 1px solid var(--cmz-acrylic-border);
}

/* 卡片毛玻璃 */
.card {
  background: var(--cmz-glass-bg);
  backdrop-filter: blur(var(--cmz-blur-md)) saturate(var(--cmz-saturate-normal));
  border: 1px solid var(--cmz-glass-border);
}
```

### 动画速度

```css
--cmz-transition-fast: 0.15s ease;
--cmz-transition-normal: 0.25s ease;
--cmz-transition-slow: 0.4s ease;
```

### 层级

```css
--cmz-z-dropdown: 100;
--cmz-z-sticky: 200;
--cmz-z-tooltip: 500;
--cmz-z-modal-backdrop: 800;
--cmz-z-modal: 900;
--cmz-z-toast: 1000;
```

### 透明度

```css
--cmz-opacity-disabled: 0.4;
--cmz-opacity-hover: 0.8;
--cmz-opacity-muted: 0.6;
```

### 边框宽度

```css
--cmz-border-width: 1px;
--cmz-border-width-bold: 2px;
```

---

## 深浅主题切换

通过 `<html>` 的 `data-theme` 属性控制：

```html
<html data-theme="dark"></html>
```

```js
// 切换
document.documentElement.setAttribute("data-theme", "dark");
// 切回浅色
document.documentElement.removeAttribute("data-theme");
```

所有组件颜色在 `[data-theme="dark"]` 下都有对应的暗色值，无需额外配置。

---

## 全局滚动条

组件库内置了精简的全局滚动条，无需额外配置。

---

## 构建

```bash
pnpm build  # 输出到 dist/
```

输出包含：

- `dist/style.css` — 样式文件
- `dist/cmzya-modern-ui.mjs` / `.cjs` — JS 入口（含所有组件）
- `dist/cmzya-modern-ui.js` — UMD
