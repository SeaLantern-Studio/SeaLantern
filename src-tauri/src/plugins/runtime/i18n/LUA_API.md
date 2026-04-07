# i18n Lua 接口说明

本文档说明插件运行时暴露的 [`sl.i18n`](./mod.rs) Lua 接口，用于查询当前语言、读取翻译、监听语言切换，以及注册插件自己的国际化资源。

## 接口总览

| Lua 接口                                          | 参数                                                                     | 返回值                  | 说明                                                                      | 对应实现                                          |
| ------------------------------------------------- | ------------------------------------------------------------------------ | ----------------------- | ------------------------------------------------------------------------- | ------------------------------------------------- |
| `sl.i18n.getLocale()`                             | 无                                                                       | `string`                | 获取当前应用语言代码，例如 `zh-CN`、`en-US`                               | [`query::get_locale()`](./query.rs:5)             |
| `sl.i18n.t(key, options?)`                        | `key: string`，`options?: table<string, string>`                         | `string`                | 按 key 获取翻译文本；当提供 `options` 时会进行 `{name}` 风格变量替换      | [`query::translate()`](./query.rs:9)              |
| `sl.i18n.hasTranslation(key, locale?)`            | `key: string`，`locale?: string`                                         | `boolean`               | 判断指定 key 是否存在翻译；不传 `locale` 时使用当前语言，带 fallback 行为 | [`query::has_translation()`](./query.rs:26)       |
| `sl.i18n.tOrDefault(key, defaultValue, options?)` | `key: string`，`defaultValue: string`，`options?: table<string, string>` | `string`                | 若 key 存在翻译则返回翻译结果，否则返回默认值                             | [`query::t_or_default()`](./query.rs:40)          |
| `sl.i18n.getAllTranslations()`                    | 无                                                                       | `table<string, string>` | 获取当前语言下可读到的全部翻译项，包含宿主与插件翻译的合并结果            | [`query::get_all_translations()`](./query.rs:73)  |
| `sl.i18n.getTranslations(locale)`                 | `locale: string`                                                         | `table<string, string>` | 获取指定语言下可读到的全部翻译项                                          | [`query::get_translations()`](./query.rs:77)      |
| `sl.i18n.getAvailableLocales()`                   | 无                                                                       | `table<number, string>` | 获取当前可用语言列表                                                      | [`query::get_available_locales()`](./query.rs:81) |
| `sl.i18n.tp(pluginId, key)`                       | `pluginId: string`，`key: string`                                        | `string`                | 读取指定插件命名空间下的翻译，相当于查询 `plugins.{pluginId}.{key}`       | [`query::translate_plugin()`](./query.rs:90)      |
| `sl.i18n.onLocaleChange(callback)`                | `callback: function(newLocale: string)`                                  | `integer`               | 注册语言切换监听器，返回回调 id，用于后续取消监听                         | [`events::on_locale_change()`](./events.rs:7)     |
| `sl.i18n.offLocaleChange(callbackId)`             | `callbackId: integer`                                                    | `boolean`               | 取消指定 id 的语言切换监听器，成功返回 `true`                             | [`events::off_locale_change()`](./events.rs:28)   |
| `sl.i18n.registerLocale(locale, displayName)`     | `locale: string`，`displayName: string`                                  | `nil`                   | 注册插件提供的新语言及其显示名称                                          | [`write::register_locale()`](./write.rs:16)       |
| `sl.i18n.addTranslations(locale, entries)`        | `locale: string`，`entries: table<string, string>`                       | `nil`                   | 为插件写入指定语言的翻译项；写入时会自动添加插件命名空间前缀              | [`write::add_translations()`](./write.rs:34)      |
| `sl.i18n.removeTranslations()`                    | 无                                                                       | `nil`                   | 移除当前插件此前注册的全部翻译项                                          | [`write::remove_translations()`](./write.rs:53)   |

## 使用说明

### 1. 查询当前语言

```lua
local locale = sl.i18n.getLocale()
print(locale)
```

### 2. 读取翻译

```lua
local text = sl.i18n.t("plugins.my-plugin.menu.title")
print(text)
```

带变量替换：

```lua
local text = sl.i18n.t("plugins.my-plugin.welcome", {
  name = "Steve"
})
print(text)
```

### 3. 查询翻译是否存在

```lua
if sl.i18n.hasTranslation("plugins.my-plugin.menu.title") then
  print("translation exists")
end

if sl.i18n.hasTranslation("plugins.my-plugin.menu.title", "en-US") then
  print("translation exists in en-US")
end
```

### 4. 读取翻译并提供默认值

```lua
local text = sl.i18n.tOrDefault("plugins.my-plugin.menu.title", "Fallback Title")
print(text)
```

### 5. 按 locale 获取翻译表

```lua
local zh = sl.i18n.getTranslations("zh-CN")
print(zh["app.title"])
```

### 6. 使用插件命名空间快捷查询

```lua
local text = sl.i18n.tp("my-plugin", "menu.title")
print(text)
```

### 7. 监听语言切换

```lua
local callbackId = sl.i18n.onLocaleChange(function(newLocale)
  print("locale changed:", newLocale)
end)

sl.i18n.offLocaleChange(callbackId)
```

### 8. 注册并写入插件翻译

```lua
sl.i18n.registerLocale("en-US", "English (US)")
sl.i18n.addTranslations("en-US", {
  ["menu.title"] = "Plugin Menu",
  ["welcome"] = "Hello, {name}!"
})
```

注意：

- [`sl.i18n.addTranslations()`](./mod.rs:112) 写入的 key 会在 Rust 层自动转换为 `plugins.{plugin_id}.{key}`。
- 例如插件写入 `menu.title`，实际存储 key 为 `plugins.<plugin_id>.menu.title`。
- 读取时可以：
  - 直接用完整 key 调用 [`sl.i18n.t()`](./mod.rs:27)
  - 或使用 [`sl.i18n.tp()`](./mod.rs:91) 通过插件 id + 短 key 查询
- 这样不会污染宿主已有的 i18n key 空间。

## 安全限制

当前写接口已带有基础安全校验，主要包括：

| 限制项            | 规则                                       | 对应实现                                               |
| ----------------- | ------------------------------------------ | ------------------------------------------------------ |
| locale 格式       | 仅允许如 `en`、`en-US`、`zh-CN`、`sr-Latn` | [`validate_locale_input()`](./write.rs:88)             |
| locale 长度       | 1 到 32 个字符                             | [`MAX_LOCALE_LEN`](./write.rs:9)                       |
| display name 长度 | 1 到 64 个字符，且不能包含控制字符         | [`validate_display_name()`](./write.rs:105)            |
| 翻译 key 字符集   | 仅允许字母、数字、`.`、`-`、`_`、`:`       | [`validate_translation_key_input()`](./write.rs:123)   |
| 翻译 key 长度     | 1 到 128 个字符                            | [`MAX_TRANSLATION_KEY_LEN`](./write.rs:11)             |
| 翻译 value 长度   | 最多 4000 个字符，且不能包含空字符         | [`validate_translation_value()`](./write.rs:140)       |
| 单次写入条数      | 最多 500 条                                | [`MAX_TRANSLATION_ENTRIES_PER_CALL`](./write.rs:13)    |
| 单插件总翻译条数  | 最多 5000 条                               | [`enforce_plugin_translation_quota()`](./write.rs:157) |
| 插件命名空间隔离  | 自动前缀化为 `plugins.{plugin_id}.`        | [`plugin_i18n_namespace()`](./common.rs:67)            |

## 备注

- [`sl.i18n.getAllTranslations()`](./mod.rs:76) 返回的是当前语言下可访问到的翻译集合。
- [`sl.i18n.getTranslations(locale)`](./mod.rs:88) 可用于调试、导出或按语言预览翻译数据。
- 插件监听器在最后一个回调移除时会自动释放底层 token，插件卸载时也会自动清理相关 i18n 资源。
