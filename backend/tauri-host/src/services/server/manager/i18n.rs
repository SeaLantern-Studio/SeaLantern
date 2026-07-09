use crate::services::global::i18n_service;
use std::collections::HashMap;

pub(super) fn manager_t(key: &str) -> String {
    i18n_service().t(key)
}

pub(super) fn manager_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn manager_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

pub(super) fn manager_t3(
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

pub(super) fn manager_t5(
    key: &str,
    a: impl Into<String>,
    b: impl Into<String>,
    c: impl Into<String>,
    d: impl Into<String>,
    e: impl Into<String>,
) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    m.insert("2".to_string(), c.into());
    m.insert("3".to_string(), d.into());
    m.insert("4".to_string(), e.into());
    i18n_service().t_with_options(key, &m)
}
