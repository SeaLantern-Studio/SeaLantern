//! 服务器建服和导入流程

mod create;
mod existing;
mod import_copy;
mod modpack;
mod shared;

use crate::models::server::{
    AddExistingServerRequest, CreateServerRequest, ImportModpackRequest, ImportServerRequest,
    ServerInstance,
};

use super::ServerManager;

/// 新建服务器记录
///
/// 这里假设启动文件已经准备好，只负责生成实例并写入列表
pub(super) fn create_server(
    manager: &ServerManager,
    req: CreateServerRequest,
) -> Result<ServerInstance, String> {
    create::create_server(manager, req)
}

/// 复制已有服务端目录并导入
///
/// # Parameters
///
/// - `manager`: 服务器管理器
/// - `req`: 导入请求
pub(super) fn import_server(
    manager: &ServerManager,
    req: ImportServerRequest,
) -> Result<ServerInstance, String> {
    import_copy::import_server(manager, req)
}

/// 导入整合包并创建服务器记录
///
/// 这里会处理压缩包、整合包目录和自定义启动方式
pub(super) fn import_modpack(
    manager: &ServerManager,
    req: ImportModpackRequest,
) -> Result<ServerInstance, String> {
    modpack::import_modpack(manager, req)
}

/// 接入已经存在的服务器目录
///
/// # Parameters
///
/// - `manager`: 服务器管理器
/// - `req`: 接入请求
pub(super) fn add_existing_server(
    manager: &ServerManager,
    req: AddExistingServerRequest,
) -> Result<ServerInstance, String> {
    existing::add_existing_server(manager, req)
}
