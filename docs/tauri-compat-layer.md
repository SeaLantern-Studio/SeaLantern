# Tauri 兼容层

后端做了大重构（`interface` 契约 / `application` 装配 / `core`+`extra`+`infra` 积木分层），Tauri adapter 层只对接了 `instance` 和 `system` 两个 service。前端 `src/api/*.ts` 仍使用旧命令名和旧参数形态。

兼容层在 `src-tauri/src/adapter/tauri/commands/compat/` 下注册前端旧命令名作为 Tauri 命令，内部做参数/响应适配后调用新 service，让**前端零改动**即可对接新后端。

## 结构

```
src-tauri/src/adapter/tauri/commands/compat/
  mod.rs                # 子模块声明
  error.rs              # 跨域错误映射（instance_err → system_err）
  models.rs             # 前端形态的请求/响应 DTO（serde 结构体）
  adapter.rs            # 纯转换函数（Instance↔前端、SystemSnapshot↔前端）+ 单测
  instance_compat.rs    # src/api/server.ts 对应的兼容命令（20 条）
  system_compat.rs      # src/api/system.ts 对应的兼容命令（8 条）
```

## 命令映射表（28 条）

### 可用（10 条）

后端已对接，立即可用：

| 兼容命令               | 后端调用                 | 说明                                |
| ---------------------- | ------------------------ | ----------------------------------- |
| `create_server`        | `instance.create`        | 参数适配：生成 UUID、目录、时间戳   |
| `add_existing_server`  | `instance.create`        | 参数适配：directory=server_path     |
| `get_server_list`      | `instance.list`          | 响应适配：Instance→前端形态         |
| `delete_server`        | `instance.delete`        | 仅改名                              |
| `update_server_name`   | `instance.rename`        | 仅改名                              |
| `update_server_path`   | `instance.update_path`   | 丢弃 jarPath/startupMode            |
| `get_system_info`      | `system.system_snapshot` | 响应适配：网络重整形                |
| `get_default_run_path` | —                        | 兼容原生：返回 app_data_dir/servers |
| `open_file`            | —                        | 兼容原生：tauri-plugin-opener       |
| `open_folder`          | —                        | 兼容原生：tauri-plugin-opener       |

### 前向就绪但暂 Unsupported（5 条）

兼容命令已就绪，但后端生命周期未接 Daemon，待 Phase 2 自动生效：

| 兼容命令                    | 后端调用                                   | 原因                             |
| --------------------------- | ------------------------------------------ | -------------------------------- |
| `start_server`              | `instance.start`                           | 后端返 Unsupported               |
| `stop_server`               | `instance.stop`                            | 后端返 Unsupported               |
| `force_stop_server`         | `instance.force_stop`                      | 后端返 Unsupported（token 丢弃） |
| `get_server_status`         | `instance.status`                          | 后端返 Unsupported               |
| `get_server_resource_usage` | `instance.status` + `system.process_usage` | 跨域，依赖 status                |

### 显式 Unsupported（13 条）

后端能力未装配，返回 `Unsupported` 错误，绝不静默 no-op：

`import_server`、`import_modpack`、`parse_server_core_type`、`scan_startup_candidates`、`collect_copy_conflicts`、`copy_directory_contents`、`prepare_force_stop_server`、`send_command`、`get_server_logs`、`validate_server_path`、`get_safe_mode_status`、`test_ipv6_connectivity`、`remove_file`

## ID / 目录 / 时间戳生成策略

- **ID**：`uuid::Uuid::new_v4()`，唯一性优先
- **目录**：`get_app_data_dir().join("servers").join(&id)`，复用跨平台定位
- **created_at**：`SystemTime::now().duration_since(UNIX_EPOCH).as_secs()`

## 已知限制（第一阶段）

1. **Docker/浏览器模式仍破**：`httpInvoke` 走 `/api/{旧命令名}`，server HTTP 适配器无对应路由。本兼容层只覆盖原生 Tauri 模式
2. **生命周期 5 命令报 Unsupported**：后端未接 Daemon，兼容命令已就绪待 Phase 2
3. **`get_server_resource_usage` 字段不全**：cpu.name/count 与 disk 在 ProcessResourceUsage 中缺失，留空/零值占位
4. **`update_server_path` 部分功能**：只改目录，丢弃 jarPath/startupMode
5. **`create_server` 的 `core_version` 留空**：待 Phase 2 由 provisioning 补

## Phase 2 后端模块装配优先级

1. Instance 生命周期（Daemon 接入）— 解锁 start/stop/force_stop/status
2. Java 管理（`crates/extra/src/java/`）
3. Config / Settings（`crates/extra/src/config/sealantern/`）
4. Provisioning（import/modpack/scan/parse）
5. Plugin（`crates/extra/src/app_plugin/` + `market/`）
6. Player（`crates/core/src/instance/player.rs`）
7. Backup（后端待建）
8. Downloader（`crates/infra/src/download/`）
9. Logging / send_command / get_server_logs（接 `server/src/rpc/service/console.rs`）
10. Update / Tunnel / Online（`crates/extra/src/update/`、`online/`）
