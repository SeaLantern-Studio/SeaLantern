use std::collections::HashMap;

pub const SUPPORTED_LOCALES: &[&str] = &[
    "zh-CN", "en-US", "zh-TW", "de-DE", "es-ES", "fr-FA", "ja-JP", "ko-KR", "ru-RU", "vi-VN",
];

pub fn embedded_table(locale_id: &str) -> HashMap<String, String> {
    let json: &str = match locale_id {
        "zh-CN" => include_str!("../locales/zh-CN.json"),
        "en-US" => include_str!("../locales/en-US.json"),
        "zh-TW" => include_str!("../locales/zh-TW.json"),
        "de-DE" => include_str!("../locales/de-DE.json"),
        "es-ES" => include_str!("../locales/es-ES.json"),
        "fr-FA" => include_str!("../locales/fr-FA.json"),
        "ja-JP" => include_str!("../locales/ja-JP.json"),
        "ko-KR" => include_str!("../locales/ko-KR.json"),
        "ru-RU" => include_str!("../locales/ru-RU.json"),
        "vi-VN" => include_str!("../locales/vi-VN.json"),
        _ => {
            eprintln!("[i18n] unsupported embedded locale: {}, falling back to zh-CN", locale_id);
            include_str!("../locales/zh-CN.json")
        }
    };

    match serde_json::from_str::<HashMap<String, String>>(json) {
        Ok(table) => table,
        Err(error) => {
            eprintln!("[i18n] failed to parse embedded locale {}: {}", locale_id, error);
            HashMap::new()
        }
    }
}
