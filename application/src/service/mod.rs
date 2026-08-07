//! 应用层服务实现模块。
//!
//! 存放各类宿主能力的默认实现（如 [`CoreInstanceService`]、[`CoreSystemService`]），
//! 实现 `interface` 的能力端口，由 `services` 装配层组装进全局容器。

mod instance;
mod settings;
mod system;

pub use instance::CoreInstanceService;
pub use settings::CoreSettingsService;
pub use system::CoreSystemService;
