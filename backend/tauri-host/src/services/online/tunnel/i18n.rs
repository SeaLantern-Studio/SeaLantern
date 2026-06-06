use crate::services::global::i18n_service;
use std::collections::HashMap;

pub(super) fn tunnel_t(key: &str) -> String {
    i18n_service().t(key)
}

pub(super) fn tunnel_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn tunnel_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn tunnel_t3(
    key: &str,
    a: impl Into<String>,
    b: impl Into<String>,
    c: impl Into<String>,
) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    m.insert("2".to_string(), c.into());
    i18n_service().t_with_options(key, &m)
}
