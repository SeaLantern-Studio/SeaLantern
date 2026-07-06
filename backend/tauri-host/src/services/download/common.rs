use crate::services::global::i18n_service;
use std::collections::HashMap;

pub(crate) fn download_t(key: &str) -> String {
    i18n_service().t(key)
}

pub(crate) fn download_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}
