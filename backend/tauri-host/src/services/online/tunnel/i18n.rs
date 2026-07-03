use super::super::host;

pub(super) fn tunnel_t(key: &str) -> String {
    host::tunnel_translate(key)
}

pub(super) fn tunnel_t1(key: &str, a: impl Into<String>) -> String {
    host::tunnel_translate_with_args(key, &[a.into()])
}

pub(super) fn tunnel_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    host::tunnel_translate_with_args(key, &[a.into(), b.into()])
}

pub(super) fn tunnel_t3(
    key: &str,
    a: impl Into<String>,
    b: impl Into<String>,
    c: impl Into<String>,
) -> String {
    host::tunnel_translate_with_args(key, &[a.into(), b.into(), c.into()])
}
