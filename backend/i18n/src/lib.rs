// License: GPL-3.0-only. Copyright (C) SeaLantern Studio.
//! Embedded locale tables and runtime i18n service primitives.

mod embedded;
mod service;

pub use embedded::{embedded_table, SUPPORTED_LOCALES};
pub use service::{I18nService, LocaleCallbackToken};
