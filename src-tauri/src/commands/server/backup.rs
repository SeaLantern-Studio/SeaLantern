use sealantern_extra::backup::{BackupItem, BackupSettings, CreateBackupRequest};

/// 获取备份列表
#[tauri::command]
pub async fn get_backup_list(server_id: String) -> Result<Vec<BackupItem>, String> {
    sealantern_extra::backup::get_backup_list(server_id)
        .await
        .map_err(|e| e.to_string())
}

/// 创建备份
#[tauri::command]
pub async fn create_backup(request: CreateBackupRequest) -> Result<BackupItem, String> {
    // 从服务器ID获取服务器目录
    // 注意：这里需要从应用状态获取服务器目录和运行状态
    // 临时实现，实际使用时需要完善
    
    // TODO: 从应用状态获取:
    // 1. 服务器目录路径
    // 2. 服务器运行状态检查函数
    
    // 临时返回错误，提示需要在应用状态中实现
    Err("需要从应用状态获取服务器信息，请完善实现".to_string())
}

/// 删除备份
#[tauri::command]
pub async fn delete_backup(backup_id: String) -> Result<(), String> {
    sealantern_extra::backup::delete_backup(backup_id)
        .await
        .map_err(|e| e.to_string())
}

/// 恢复备份
#[tauri::command]
pub async fn restore_backup(backup_id: String) -> Result<(), String> {
    // 同 create_backup，需要从应用状态获取服务器信息
    Err("需要从应用状态获取服务器信息，请完善实现".to_string())
}

/// 获取备份设置
#[tauri::command]
pub async fn get_backup_settings(server_id: String) -> Result<BackupSettings, String> {
    sealantern_extra::backup::get_backup_settings(server_id)
        .await
        .map_err(|e| e.to_string())
}

/// 更新备份设置
#[tauri::command]
pub async fn update_backup_settings(
    server_id: String,
    settings: BackupSettings,
) -> Result<(), String> {
    sealantern_extra::backup::update_backup_settings(server_id, settings)
        .await
        .map_err(|e| e.to_string())
}