//! sl.storage 命名空间装配入口
//!
//! 职责：
//! - 创建 Lua 表 sl.storage，并按职责挂载读写能力
//! - 保持入口轻量，不在此处堆积具体闭包实现

use super::PluginRuntime;
use mlua::Table;

mod common;
mod read;
mod write;

use common::{create_storage_table, set_storage_table, StorageContext};

impl PluginRuntime {
    pub(super) fn setup_storage_namespace(&self, sl: &Table) -> Result<(), String> {
        let storage = create_storage_table(&self.lua)?;
        let ctx = StorageContext::new(&self.data_dir, self.storage_lock.clone());

        read::register(&self.lua, &storage, &ctx)?;
        write::register(&self.lua, &storage, &ctx)?;

        set_storage_table(sl, storage)
    }
}
