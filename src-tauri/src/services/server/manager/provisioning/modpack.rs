mod build;
mod files;
mod mapping;
mod properties;
mod run_dir;
mod startup;

use std::path::Path;

use crate::models::server::{ImportModpackRequest, ServerInstance};

use super::super::common::validate_server_name;
use super::super::ServerManager;
use super::shared::write_sl_startup_config;

pub(super) fn import_modpack(
    manager: &ServerManager,
    req: ImportModpackRequest,
) -> Result<ServerInstance, String> {
    let source_path = Path::new(&req.modpack_path);
    if !source_path.exists() {
        return Err(format!("整合包路径不存在: {}", req.modpack_path));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let server_name = validate_server_name(&req.name)?;
    let run_dir = run_dir::resolve_modpack_run_dir(&req.run_path)?;
    files::prepare_modpack_files(source_path, &run_dir)?;
    let startup = startup::resolve_modpack_startup_selection(&req, source_path, &run_dir)?;

    let data_dir = manager.data_dir_value()?;
    mapping::save_modpack_run_mapping(&data_dir, &id, &server_name, &req, &run_dir, &startup)?;

    let port =
        properties::ensure_server_properties_for_import(&run_dir, req.port, req.online_mode)?;
    write_sl_startup_config(
        &run_dir,
        req.max_memory,
        req.min_memory,
        req.jvm_args.clone(),
        req.cpu_policy.clone(),
        req.jvm_preset.clone(),
    )?;
    let server =
        build::build_modpack_server_instance(id, server_name, req, &run_dir, startup, port);

    println!(
        "创建服务器实例: id={}, path={}, startup_path={}",
        server.id,
        server.path,
        server.jar_path().unwrap_or_default()
    );

    manager.lock_servers()?.push(server.clone());
    manager.save()?;
    Ok(server)
}
