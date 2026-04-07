//! 后端内置文案：`src-tauri/locales/<locale>.json`（扁平 `key -> string`），编译期嵌入。
//! 与 `utils/constants::SUPPORTED_LOCALES`、前端 `src/language/*.json` 语言代码对齐。
//! 部分语言仅含 `tunnel.*` 等增量键，其余键由 `I18nService::t` 回退到 zh-CN。

use std::collections::HashMap;

pub fn embedded_table(locale_id: &str) -> HashMap<String, String> {
    let json: &str = match locale_id {
        "zh-CN" => include_str!("../../locales/zh-CN.json"),
        "en-US" => include_str!("../../locales/en-US.json"),
        "zh-TW" => include_str!("../../locales/zh-TW.json"),
        "de-DE" => include_str!("../../locales/de-DE.json"),
        "es-ES" => include_str!("../../locales/es-ES.json"),
        "fr-FA" => include_str!("../../locales/fr-FA.json"),
        "ja-JP" => include_str!("../../locales/ja-JP.json"),
        "ko-KR" => include_str!("../../locales/ko-KR.json"),
        "ru-RU" => include_str!("../../locales/ru-RU.json"),
        "vi-VN" => include_str!("../../locales/vi-VN.json"),
        _ => panic!("unsupported embedded locale: {locale_id}"),
    };
    serde_json::from_str(json).unwrap_or_else(|e| panic!("locale {locale_id} json: {e}"))
}
