mod embedded;
mod service;

pub use embedded::{embedded_table, SUPPORTED_LOCALES};
pub use service::{I18nService, LocaleCallbackToken};
