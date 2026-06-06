use super::common::{CommandHandler, RegistryBuilder, parse_params};
use super::requests::{
    BanPlayerRequest, ExportLogsRequest, KickPlayerRequest, PlayerActionRequest, ServerPathRequest,
};
use crate::commands::server::players as player_commands;
use serde_json::Value;
pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("get_whitelist", handle_get_whitelist as CommandHandler);
    builder.register("get_banned_players", handle_get_banned_players as CommandHandler);
    builder.register("get_ops", handle_get_ops as CommandHandler);
    builder.register("add_to_whitelist", handle_add_to_whitelist as CommandHandler);
    builder.register(
        "remove_from_whitelist",
        handle_remove_from_whitelist as CommandHandler,
    );
    builder.register("ban_player", handle_ban_player as CommandHandler);
    builder.register("unban_player", handle_unban_player as CommandHandler);
    builder.register("add_op", handle_add_op as CommandHandler);
    builder.register("remove_op", handle_remove_op as CommandHandler);
    builder.register("kick_player", handle_kick_player as CommandHandler);
    builder.register("export_logs", handle_export_logs as CommandHandler);
}

fn handle_get_whitelist(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ServerPathRequest = parse_params(params)?;
        let result = player_commands::get_whitelist(req.server_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_get_banned_players(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ServerPathRequest = parse_params(params)?;
        let result = player_commands::get_banned_players(req.server_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_get_ops(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ServerPathRequest = parse_params(params)?;
        let result = player_commands::get_ops(req.server_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_add_to_whitelist(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest = parse_params(params)?;
        let result = player_commands::add_to_whitelist(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_remove_from_whitelist(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest = parse_params(params)?;
        let result = player_commands::remove_from_whitelist(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_ban_player(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: BanPlayerRequest = parse_params(params)?;
        let result = player_commands::ban_player(req.server_id, req.name, req.reason)?;
        Ok(Value::String(result))
    })
}

fn handle_unban_player(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest = parse_params(params)?;
        let result = player_commands::unban_player(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_add_op(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest = parse_params(params)?;
        let result = player_commands::add_op(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_remove_op(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PlayerActionRequest = parse_params(params)?;
        let result = player_commands::remove_op(req.server_id, req.name)?;
        Ok(Value::String(result))
    })
}

fn handle_kick_player(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: KickPlayerRequest = parse_params(params)?;
        let result = player_commands::kick_player(req.server_id, req.name, req.reason)?;
        Ok(Value::String(result))
    })
}

fn handle_export_logs(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ExportLogsRequest = parse_params(params)?;
        player_commands::export_logs(req.logs, req.save_path)?;
        Ok(Value::Null)
    })
}
