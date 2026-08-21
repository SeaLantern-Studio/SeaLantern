# CmzYa Modern UI

轻量级 Vue 3 组件库，CSS 变量驱动，支持深浅主题、系统主题、毛玻璃和动画全局开关。

## 目录

- [安装](#安装)
- [快速使用](#快速使用)
- [组件列表](#组件列表)
- [CSS 变量](#css-变量)
- [交互反馈语言](#交互反馈语言)
- [全局主题控制](#全局主题控制)
- [构建](#构建)
- [License](#license)

## 安装

```bash
pnpm add cmzya-modern-ui
```

## 快速使用

```vue
<script setup>
import { Cmz_Button, Cmz_Card } from "cmzya-modern-ui";
import "cmzya-modern-ui/style.css";
</script>

<template>
  <Cmz_Button>按钮</Cmz_Button>
</template>
```

## 组件列表

共 21 个组件，另有 4 个保留底层能力的薄包装组件，见[全局主题控制](#全局主题控制)。

### Cmz_Accordion

手风琴折叠容器，管理多个面板的展开/折叠状态。

| 属性       | 类型       | 默认值  | 说明                   |
| ---------- | ---------- | ------- | ---------------------- |
| modelValue | `string[]` | `[]`    | 当前展开的面板 ID 数组 |
| multiple   | `boolean`  | `false` | 是否允许多开           |

事件: `update:modelValue`

---

### Cmz_AccordionPanel

单个折叠面板，必须作为 Cmz_Accordion 的子级使用。

| 属性     | 类型      | 默认值  | 说明         |
| -------- | --------- | ------- | ------------ |
| id       | `string`  | —       | 面板唯一标识 |
| title    | `string`  | —       | 面板标题     |
| disabled | `boolean` | `false` | 是否禁用     |

插槽: `title`（自定义标题，作用域暴露 `open` 状态）、`default`（内容）

---

### Cmz_Badge

胶囊徽标，支持文本、圆点、可关闭三种模式。

| 属性     | 类型                             | 默认值     | 说明               |
| -------- | -------------------------------- | ---------- | ------------------ |
| text     | `string`                         | —          | 徽标文本           |
| color    | `string`                         | —          | 自定义颜色         |
| size     | `'small' \| 'medium' \| 'large'` | `'medium'` | 尺寸               |
| closable | `boolean`                        | `false`    | 是否可关闭         |
| maxWidth | `number \| string`               | —          | 最大宽度           |
| dot      | `boolean`                        | `false`    | 是否显示圆点模式   |
| pulse    | `boolean`                        | `false`    | 圆点是否有脉冲动画 |

事件: `close`

---

### Cmz_Button

按钮组件，支持三种变体。

| 属性     | 类型                              | 默认值     | 说明           |
| -------- | --------------------------------- | ---------- | -------------- |
| variant  | `'solid' \| 'outline' \| 'ghost'` | `'solid'`  | 按钮变体       |
| size     | `'sm' \| 'md' \| 'lg'`            | `'md'`     | 尺寸           |
| color    | `string`                          | —          | 自定义颜色     |
| type     | `'button' \| 'submit' \| 'reset'` | `'button'` | 按钮类型       |
| disabled | `boolean`                         | `false`    | 是否禁用       |
| loading  | `boolean`                         | `false`    | 是否加载中     |
| iconOnly | `boolean`                         | `false`    | 是否仅图标模式 |

插槽: `default`（按钮内容）

CSS 变量: `--cmz-btn-hover-lift`（默认 `-1px`）控制 solid / outline 按钮 hover 的上浮距离，设为 `0` 关掉位移只留配色变化

---

### Cmz_Card

卡片容器组件。

| 属性      | 类型                                            | 默认值    | 说明                                                                    |
| --------- | ----------------------------------------------- | --------- | ----------------------------------------------------------------------- |
| title     | `string`                                        | —         | 卡片标题                                                                |
| subtitle  | `string`                                        | —         | 卡片副标题                                                              |
| hoverable | `boolean`                                       | `false`   | 是否启用悬停效果（仅视觉，不设置 cursor，需要指针手势由消费方自行添加） |
| padding   | `'none' \| 'xs' \| 'sm' \| 'md' \| 'lg'`        | `'md'`    | 内边距大小                                                              |
| variant   | `'solid' \| 'glass' \| 'outline' \| 'elevated'` | `'solid'` | 卡片变体                                                                |

插槽: `default`（内容）、`header`（头部）、`actions`（操作区）、`footer`（底部）

CSS 变量: `--cmz-card-hover-lift`（默认 `-2px`）控制 hoverable 卡片的悬停抬起量，设为 `0` 关掉位移；elevated 变体自动使用双倍落差。hoverable 同时压掉卡内按钮的 `--cmz-btn-hover-lift`，避免二级上浮

---

### Cmz_ColorPicker

取色器组件，弹层式：点击 trigger 弹出面板，内含饱和度/明度二维面板、色相条、HEX 输入框和预设色板，全部交互均可键盘操作。输出统一规范化为小写 `#rrggbb`，不处理 alpha 通道。

| 属性            | 类型                   | 默认值           | 说明                                                       |
| --------------- | ---------------------- | ---------------- | ---------------------------------------------------------- |
| modelValue      | `string`               | —                | 当前颜色，`#rrggbb` 格式                                   |
| disabled        | `boolean`              | `false`          | 是否禁用                                                   |
| label           | `string`               | —                | 触发器上方标签文字                                         |
| placeholder     | `string`               | `'Pick a color'` | 未选色时展示的文字                                         |
| presetColors    | `string[]`             | 内置色板         | 自定义预设色板，空数组隐藏色板区                           |
| size            | `'sm' \| 'md' \| 'lg'` | `'md'`           | 触发器尺寸                                                 |
| id              | `string`               | —                | 原生元素 ID                                                |
| name            | `string`               | —                | 原生表单提交名                                             |
| ariaDescribedby | `string`               | —                | 自定义描述元素 ID；在 FormField 内与提示或错误 ID 合并去重 |
| invalid         | `boolean`              | `false`          | 校验不通过态                                               |
| required        | `boolean`              | `false`          | 必填态，通过 `name` 提交时映射为 `aria-required`           |

事件: `update:modelValue` 拖动/输入过程中持续触发，`change` 在输入确认或点击预设色板时触发

---

### Cmz_Console

终端控制台组件，支持日志显示、输入、过滤、Tab 补全。日志行自动解析前缀标签：行首裸时间戳（如 `23:41:21`）、方括号时间（`[23:41:21]`）、元数据括号（`[main]`）与日志级别括号（`[INFO]`）分别渲染为时间、元数据、级别胶囊，级别同时进入内置筛选器。

| 属性             | 类型               | 默认值    | 说明                   |
| ---------------- | ------------------ | --------- | ---------------------- |
| lines            | `ConsoleLine[]`    | —         | 日志行数据             |
| showTimestamps   | `boolean`          | `false`   | 是否显示时间戳         |
| autoScroll       | `boolean`          | `true`    | 是否自动滚动到底部     |
| maxLines         | `number`           | `5000`    | 最大保留行数           |
| height           | `string`           | `'400px'` | 控制台高度             |
| readonly         | `boolean`          | `false`   | 是否只读模式           |
| placeholder      | `string`           | `''`      | 空状态占位文本         |
| selectionColor   | `string`           | `''`      | 自定义选择高亮颜色     |
| history          | `string[]`         | `[]`      | 历史命令记录           |
| completionTree   | `CompletionNode[]` | `[]`      | 补全树数据             |
| completionMd     | `string`           | `''`      | 补全指令 Markdown 内容 |
| enableCompletion | `boolean`          | `true`    | 是否启用 Tab 补全      |
| enableHistory    | `boolean`          | `true`    | 是否启用上下键历史     |

事件: `command`（提交命令）、`clear`（清空）、`update:lines`（支持 `v-model:lines` 双向绑定）

---

### Cmz_Divider

分隔线组件。

| 属性        | 类型                              | 默认值         | 说明           |
| ----------- | --------------------------------- | -------------- | -------------- |
| orientation | `'horizontal' \| 'vertical'`      | `'horizontal'` | 方向           |
| label       | `string`                          | —              | 分隔线标签文本 |
| variant     | `'solid' \| 'dashed' \| 'dotted'` | `'solid'`      | 线型           |
| thickness   | `'thin' \| 'normal' \| 'thick'`   | `'normal'`     | 线粗细         |

---

### Cmz_Dropzone

文件拖放区组件（桌面端可提取路径）。

| 属性           | 类型       | 默认值                                        | 说明               |
| -------------- | ---------- | --------------------------------------------- | ------------------ |
| modelValue     | `string`   | `''`                                          | 绑定值（当前路径） |
| label          | `string`   | `''`                                          | 主标签文本         |
| subLabel       | `string`   | `''`                                          | 副标签文本         |
| badge          | `string`   | `''`                                          | 右上角徽标文本     |
| disabled       | `boolean`  | `false`                                       | 是否禁用           |
| loading        | `boolean`  | `false`                                       | 是否加载中         |
| isDragging     | `boolean`  | —                                             | 拖拽状态（受控）   |
| acceptFolders  | `boolean`  | `true`                                        | 是否接受文件夹     |
| acceptFiles    | `boolean`  | `true`                                        | 是否接受文件       |
| fileExtensions | `string[]` | `['.zip', '.tar', '.tar.gz', '.tgz', '.jar']` | 允许的文件扩展名   |
| placeholder    | `string`   | `''`                                          | 占位文本           |
| clearable      | `boolean`  | `true`                                        | 是否可清除         |
| multiple       | `boolean`  | `false`                                       | 是否支持多文件     |

事件: `update:modelValue`、`drop`、`dropMultiple`、`clear`、`click`、`error`、`update:isDragging`、`files`、`paths`。`files` 和路径事件只返回通过类型配置校验的数据。

插槽: `icon`、`title`、`subtitle`、`footer`、`buttons`

---

### Cmz_Field

表单字段组件，整合文本、数字、选择三种变体。

| 属性            | 类型                             | 默认值   | 说明                                                       |
| --------------- | -------------------------------- | -------- | ---------------------------------------------------------- |
| modelValue      | `string \| number`               | `''`     | 绑定值                                                     |
| variant         | `'text' \| 'number' \| 'select'` | `'text'` | 字段变体                                                   |
| placeholder     | `string`                         | `''`     | 占位文本                                                   |
| label           | `string`                         | —        | 字段标签                                                   |
| disabled        | `boolean`                        | `false`  | 是否禁用                                                   |
| options         | `SelectOption[]`                 | —        | 选择项（select 变体）                                      |
| min             | `number`                         | —        | 最小值（number 变体）                                      |
| max             | `number`                         | —        | 最大值（number 变体）                                      |
| step            | `number`                         | `1`      | 步进值（number 变体）                                      |
| id              | `string`                         | —        | 原生元素 ID                                                |
| name            | `string`                         | —        | 原生表单提交名                                             |
| ariaDescribedby | `string`                         | —        | 自定义描述元素 ID；在 FormField 内与提示或错误 ID 合并去重 |
| invalid         | `boolean`                        | `false`  | 校验不通过态                                               |
| required        | `boolean`                        | `false`  | 是否必填；文本与数字变体接入原生表单校验                   |

事件: `update:modelValue`

---

### Cmz_FormField

表单字段容器，含标签、错误提示、帮助文本。

| 属性          | 类型              | 默认值  | 说明                                             |
| ------------- | ----------------- | ------- | ------------------------------------------------ |
| label         | `string`          | —       | 字段标签                                         |
| required      | `boolean`         | `false` | 是否必填（显示星号）                             |
| error         | `string`          | —       | 错误信息                                         |
| hint          | `string`          | —       | 帮助提示文本                                     |
| labelPosition | `'top' \| 'left'` | `'top'` | 标签位置                                         |
| id            | `string`          | —       | 内部控件 ID 前缀，实际控件 ID 为 `${id}-control` |

插槽: `default`（字段内容）

---

### Cmz_Input

输入框组件，支持单行/多行。

| 属性               | 类型                                             | 默认值       | 说明                                                       |
| ------------------ | ------------------------------------------------ | ------------ | ---------------------------------------------------------- |
| modelValue         | `string`                                         | `''`         | 绑定值                                                     |
| placeholder        | `string`                                         | `''`         | 占位文本                                                   |
| label              | `string`                                         | —            | 标签文本                                                   |
| disabled           | `boolean`                                        | `false`      | 是否禁用                                                   |
| maxlength          | `number`                                         | —            | 最大字符数                                                 |
| multiline          | `boolean`                                        | `false`      | 是否多行文本                                               |
| rows               | `number`                                         | `4`          | 多行时行数                                                 |
| resize             | `'none' \| 'vertical' \| 'horizontal' \| 'both'` | `'vertical'` | 多行时调整大小方式                                         |
| type               | `string`                                         | `'text'`     | 输入类型（单行）                                           |
| min                | `number`                                         | —            | 数字最小值                                                 |
| max                | `number`                                         | —            | 数字最大值                                                 |
| step               | `number`                                         | —            | 数字步进                                                   |
| hideNumberControls | `boolean`                                        | `true`       | 是否隐藏数字上下箭头                                       |
| id                 | `string`                                         | —            | 原生元素 ID                                                |
| name               | `string`                                         | —            | 原生表单提交名                                             |
| ariaDescribedby    | `string`                                         | —            | 自定义描述元素 ID；在 FormField 内与提示或错误 ID 合并去重 |
| invalid            | `boolean`                                        | `false`      | 校验不通过态                                               |
| required           | `boolean`                                        | `false`      | 是否必填；接入原生表单校验                                 |

事件: `update:modelValue`

插槽: `prefix`（前缀）、`suffix`（后缀）

---

### Cmz_Markdown

Markdown 渲染组件，支持代码高亮、警示框、链接卡片、列表网格布局。

| 属性            | 类型                           | 默认值                                             | 说明                                       |
| --------------- | ------------------------------ | -------------------------------------------------- | ------------------------------------------ |
| content         | `string`                       | —                                                  | Markdown 原文                              |
| roundedTable    | `boolean`                      | `true`                                             | 表格是否圆角                               |
| codeHighlight   | `boolean`                      | `true`                                             | 是否启用代码语法高亮                       |
| sanitized       | `boolean`                      | `true`                                             | 已弃用的兼容属性；原始 HTML 始终按文本处理 |
| variant         | `'plain' \| 'card' \| 'glass'` | `'plain'`                                          | 容器样式                                   |
| features        | `MarkdownFeatures`             | `{ alert: true, linkCard: true, container: true }` | 启用的特殊语法                             |
| listLayout      | `'stack' \| 'grid'`            | `'stack'`                                          | 列表布局模式                               |
| listGridColumns | `number \| 'auto'`             | `3`                                                | grid 模式下列数                            |

---

### Cmz_Modal

模态框组件。

| 属性            | 类型      | 默认值    | 说明                                                      |
| --------------- | --------- | --------- | --------------------------------------------------------- |
| visible         | `boolean` | —         | 是否显示                                                  |
| title           | `string`  | —         | 弹窗标题                                                  |
| width           | `string`  | `'480px'` | 弹窗宽度                                                  |
| closeOnOverlay  | `boolean` | `true`    | 点击遮罩是否关闭                                          |
| autoClose       | `number`  | `0`       | 自动关闭毫秒数；显示期间修改会重新计时，设为 0 会取消计时 |
| showCloseButton | `boolean` | `true`    | 是否显示关闭按钮                                          |

事件: `close`、`update:visible`（支持 `v-model` 双向绑定）

插槽: `default`（内容）、`footer`（底部）

---

### Cmz_Progress

进度条组件。

| 属性        | 类型      | 默认值 | 说明             |
| ----------- | --------- | ------ | ---------------- |
| value       | `number`  | —      | 当前值           |
| max         | `number`  | `100`  | 最大值           |
| label       | `string`  | —      | 进度标签         |
| showPercent | `boolean` | `true` | 是否显示百分比   |
| color       | `string`  | —      | 自定义进度条颜色 |

---

### Cmz_Select

下拉选择组件，支持搜索、键盘导航。

| 属性            | 类型                | 默认值     | 说明                                                       |
| --------------- | ------------------- | ---------- | ---------------------------------------------------------- |
| modelValue      | `string \| number`  | —          | 绑定值                                                     |
| options         | `SelectOption[]`    | —          | 选项列表                                                   |
| label           | `string`            | —          | 选择框标签                                                 |
| placeholder     | `string`            | `'Select'` | 占位文本                                                   |
| disabled        | `boolean`           | `false`    | 是否禁用                                                   |
| searchable      | `boolean`           | `false`    | 是否可搜索                                                 |
| loading         | `boolean`           | `false`    | 是否加载中                                                 |
| maxHeight       | `string`            | `'280px'`  | 下拉最大高度，保留 px、rem、vh、calc 等有效 CSS 长度       |
| collapsed       | `boolean`           | `false`    | 是否折叠（仅图标）                                         |
| dropdownAlign   | `'left' \| 'right'` | `'left'`   | 下拉对齐方向                                               |
| dropdownWidth   | `string`            | `'200px'`  | 下拉宽度，保留任意有效 CSS 长度                            |
| icon            | `Component`         | —          | 自定义图标组件                                             |
| id              | `string`            | —          | 原生元素 ID                                                |
| name            | `string`            | —          | 原生表单提交名                                             |
| ariaDescribedby | `string`            | —          | 自定义描述元素 ID；在 FormField 内与提示或错误 ID 合并去重 |
| invalid         | `boolean`           | `false`    | 校验不通过态                                               |
| required        | `boolean`           | `false`    | 是否必填                                                   |

事件: `update:modelValue`

---

### Cmz_Spinner

加载旋转指示器。

| 属性 | 类型                   | 默认值 | 说明 |
| ---- | ---------------------- | ------ | ---- |
| size | `'sm' \| 'md' \| 'lg'` | `'md'` | 尺寸 |

---

### Cmz_TabBar

标签栏组件，样式与逻辑解耦：两种层级样式（下划线/胶囊）× 两种交互逻辑（普通切换/滚动联动），均支持垂直布局。

| 属性            | 类型                    | 默认值     | 说明                                       |
| --------------- | ----------------------- | ---------- | ------------------------------------------ |
| modelValue      | `T`                     | —          | 当前选中的 key                             |
| tabs            | `TabBarItem<T>[]`       | —          | 标签项数组                                 |
| level           | `1 \| 2`                | `1`        | 层级样式（1 下划线/2 胶囊）                |
| vertical        | `boolean`               | `false`    | 是否垂直布局                               |
| scrollSpy       | `boolean`               | `false`    | 滚动联动模式，可与任一样式组合             |
| scrollContainer | `string \| HTMLElement` | 窗口       | scrollSpy 滚动容器选择器或元素             |
| scrollOffset    | `number`                | `0`        | scrollSpy 定位偏移量，如吸顶头部高度       |
| sectionSelector | `string`                | `'#{key}'` | scrollSpy 区块选择器模板，`{key}` 为占位符 |

事件: `update:modelValue`

插槽: `extra`（额外操作区）

scrollSpy 滚动联动：激活指示随滚动容器自动定位到当前可视区块，点击标签平滑滚动到对应区块；纵向容器按垂直方向联动，区块横排的横向容器自动按水平方向联动；与 level/vertical 任意组合使用。

---

### Cmz_Toast

轻提示组件，支持队列容量控制。

| 属性       | 类型            | 默认值        | 说明             |
| ---------- | --------------- | ------------- | ---------------- |
| position   | `ToastPosition` | `'top-right'` | 显示位置         |
| maxVisible | `number`        | `5`           | 最多同时显示数量 |

**Toast 选项**（通过 `useToast()` 调用）:

| 属性        | 类型                                                       | 默认值   | 说明                         |
| ----------- | ---------------------------------------------------------- | -------- | ---------------------------- |
| title       | `string`                                                   | —        | 提示标题                     |
| description | `string`                                                   | —        | 描述文本                     |
| type        | `'success' \| 'error' \| 'warning' \| 'info' \| 'loading'` | `'info'` | 提示类型                     |
| color       | `string`                                                   | —        | 自定义颜色                   |
| duration    | `number`                                                   | `5000`   | 自动关闭时间（ms，0 为常驻） |
| closable    | `boolean`                                                  | `true`   | 是否可关闭                   |
| icon        | `Component`                                                | —        | 自定义图标                   |
| action      | `{ label, onClick }`                                       | —        | 操作按钮                     |

**队列容量与默认时长**:

```ts
const toast = useToast({ max: 5, defaultDuration: 3000 });

toast.setMax(2);
toast.setDefaultDuration(5000);
```

`useToast({ max })` 和 `toast.setMax(max)` 设置全局队列容量。新增 Toast 超限时按 FIFO 删除最旧项；容量调小时立即删除多余的最旧 Toast。`maxVisible` 只控制组件当前展示数量，不改变全局队列容量。

`useToast({ defaultDuration })` 和 `toast.setDefaultDuration(ms)` 设置默认自动关闭时长，push 时未指定 `duration` 的 Toast 按该值填充；显式传 `0` 仍是常驻提示，不会被默认值覆盖。

**CSS 变量**:

| 变量                         | 默认值  | 说明           |
| ---------------------------- | ------- | -------------- |
| `--cmz-toast-offset`         | `16px`  | 容器距视口边距 |
| `--cmz-toast-gap`            | `8px`   | Toast 条间距   |
| `--cmz-toast-max-width`      | `420px` | 单条最大宽度   |
| `--cmz-toast-slide-distance` | `12px`  | 进出场位移距离 |

进出场动画方向跟随 `position`：顶部位置从上滑入、向上飞出；底部位置从下滑入、向下飞出，配合 `move` 补位动画形成"顶入顶出"效果。

视觉上类型色以左缘渐变呈现：左侧一条饱和色带向右快速淡出，主体保持浅色，`color` 自定义色和 `--toast-color` 同样参与该渐变。

---

### Cmz_Toggle

开关组件，支持 switch 和 checkbox 两种形态。

| 属性            | 类型                     | 默认值     | 说明                                                       |
| --------------- | ------------------------ | ---------- | ---------------------------------------------------------- |
| modelValue      | `boolean`                | `false`    | 绑定值                                                     |
| variant         | `'checkbox' \| 'switch'` | `'switch'` | 开关形态                                                   |
| size            | `'sm' \| 'md' \| 'lg'`   | `'md'`     | 尺寸                                                       |
| label           | `string`                 | —          | 标签文本                                                   |
| disabled        | `boolean`                | `false`    | 是否禁用                                                   |
| indeterminate   | `boolean`                | `false`    | 是否半选状态（checkbox）                                   |
| readonly        | `boolean`                | `false`    | 是否只读                                                   |
| id              | `string`                 | —          | 原生元素 ID                                                |
| ariaDescribedby | `string`                 | —          | 自定义描述元素 ID；在 FormField 内与提示或错误 ID 合并去重 |
| invalid         | `boolean`                | `false`    | 校验不通过态                                               |
| required        | `boolean`                | `false`    | 是否必填                                                   |

事件: `update:modelValue`、`change`

手感设计：switch 滑块快出缓停（`cubic-bezier(0.2, 0, 0, 1)`），按压时变宽成胶囊模拟实体开关形变，选中态按压向左延展不溢出轨道；checkbox 勾选播放"压缩-释放"两段式按压动画，均无回弹过冲。

---

### Cmz_Tooltip

浮层提示组件，四方向自动避让。

| 属性      | 类型                                     | 默认值  | 说明           |
| --------- | ---------------------------------------- | ------- | -------------- |
| content   | `string`                                 | —       | 提示文本内容   |
| delay     | `number`                                 | —       | 显示延迟（ms） |
| placement | `'top' \| 'bottom' \| 'left' \| 'right'` | `'top'` | 弹出方向       |

插槽: `default`（触发元素）

## CSS 变量

所有样式通过 `--cmz-*` 变量驱动，修改变量即可全局换肤。三层背景体系：

- `--cmz-bg` — 页面底色
- `--cmz-bg-secondary` — 容器（卡片/弹窗）
- `--cmz-bg-tertiary` — 容器内元素（输入框/开关）

强调色统一使用 `--cmz-primary`，圆角体系使用 `--cmz-radius-*`。

推荐在应用入口显式导入公开 CSS 子路径，并在导入后覆盖变量：

```ts
import "cmzya-modern-ui/style.css";
import "./theme.css";
```

```css
:root {
  --cmz-primary: #2563eb;
  --cmz-bg: #f8fafc;
  --cmz-radius-md: 12px;
  --cmz-font-sans: system-ui, sans-serif;
}
```

常用变量分组包括颜色 `--cmz-primary`、`--cmz-surface`、`--cmz-text-*`，状态色 `--cmz-success / --cmz-warning / --cmz-error / --cmz-info / --cmz-debug / --cmz-notice`（每色配套 `--cmz-*-bg` 半透明背景，Console 级别、Toast 类型等统一复用），间距 `--cmz-space-*`，圆角 `--cmz-radius-*`，阴影 `--cmz-shadow-*`，字体 `--cmz-font-*`，层级 `--cmz-z-*` 和过渡 `--cmz-transition-fast/normal/slow`。过渡变量已含时长与平滑缓动曲线（`cubic-bezier(0.4, 0, 0.2, 1)`），组件内不再写死时长，覆盖这三个变量即可全局调整所有交互动画的速度与手感。毛玻璃材质统一走 `--cmz-acrylic-bg / --cmz-acrylic-bg-strong / --cmz-acrylic-border / --cmz-acrylic-blur / --cmz-acrylic-saturate` 一套变量，卡片、弹窗、下拉、Toast 等浮层共用，不再有独立的 glass 体系。

交互位移量同样变量化：`--cmz-btn-hover-lift`（按钮 hover 上浮，默认 `-1px`）、`--cmz-card-hover-lift`（Card hoverable 抬起，默认 `-2px`，elevated 变体自动双倍）、`--cmz-toast-slide-distance`（Toast 顶入位移，默认 `12px`）。动效规范全库统一：只允许平滑过渡曲线，禁止回弹/过冲（回归测试强制扫描）。

## 交互反馈语言

全库交互元素采用统一的按压反馈，保证手感一致：

- **表单触发器**（Select trigger、ColorPicker trigger）：按压时主色浸染背景 + 边框加深
- **面板头**（AccordionPanel）：按压时标题染主色
- **小按钮**（Toast 关闭、Badge 关闭）：按压缩放 `0.88`
- **实体感开关**（Toggle switch）：按压时滑块变宽成胶囊；checkbox 播放压缩-释放动画

所有 `:active` 反馈与 hover 态共用同一过渡曲线（`--cmz-transition-fast`），按压即响应无延迟，松开平滑回落。

## 全局主题控制

通过 `<html>` 上的 data 属性驱动，所有组件自动响应：

```html
<html data-theme="dark" data-acrylic="off" data-animation="off"></html>
```

或使用 `useTheme()` composable 编程控制：

```ts
import { useTheme } from "cmzya-modern-ui";

const { theme, acrylic, animation, toggleTheme, toggleAcrylic, toggleAnimation } = useTheme();

theme.value = "system";
```

`useTheme()` 返回模块级共享状态，多个调用方会读取和修改同一主题配置。`theme` 支持 `light`、`dark`、`system`，存储不可用时自动回退到内存状态。

`Cmz_Checkbox`、`Cmz_Switch`、`Cmz_Textarea` 和 `Cmz_StatusIndicator` 是保留底层组件能力的薄包装组件，分别默认启用 checkbox、switch、multiline 和 dot 语义。

`useToastQueue()` 仅公开只读队列快照和 `remove` 方法；业务代码应通过 `useToast()` 推送、清空或调整容量。

## 构建

```bash
pnpm build
# 输出到 dist/
# - dist/cmzya.es.js   (ESM)
# - dist/cmzya.cjs     (CommonJS)
# - dist/style.css
# - dist/types/        (TypeScript 声明)
```

## License

MIT
