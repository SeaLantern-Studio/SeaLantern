use crate::plugins::runtime::PluginRuntime;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub(crate) type SharedRuntimes = Arc<RwLock<HashMap<String, PluginRuntime>>>;

/// 创建共享运行时表
pub(crate) fn new_shared_runtimes() -> SharedRuntimes {
    Arc::new(RwLock::new(HashMap::new()))
}
