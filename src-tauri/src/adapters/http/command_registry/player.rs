use super::common::{parse_params, CommandHandler};
use super::requests::{
    BanPlayerRequest, ExportLogsRequest, KickPlayerRequest, PlayerActionRequest,
    ServerPathRequest,
};
use crate::commands::server::players as player_commands;
use serde_json::Value;
use std::collections::HashMap;

pub(super) fn register_handlers(handlers: &mut HashMap<String, CommandHandler>) {
    handlers.insert("get_whitelist".to_string(), handle_get_whitelist as CommandHandler);
    handlers.insert(
        "get_banned_players".to_string(),
        handle_get_banned_players as CommandHandler,
    );
    handlers.insert("get_ops".to_string(), handle_get_ops as CommandHandler);
    handlers.insert(
        "add_to_whitelist".to_string(),
        handle_add_to_whitelist as CommandHandler,
    );
    handlers.insert(
        "remove_from_whitelist".to_string(),
        handle_remove_from_whitelist as CommandHandler,
    );
    handlers.insert("ban_player".to_string(), handle_ban_player as CommandHandler);
    handlers.insert("unban_player".to_string(), handle_unban_player as CommandHandler);
    handlers.insert("add_op".to_string(), handle_add_op as CommandHandler);
    handlers.insert("remove_op".to_string(), handle_remove_op as CommandHandler);
    handlers.insert("kick_player".to_string(), handle_kick_player as CommandHandler);
    handlers.insert("export_logs".to_string(), handle_export_logs as CommandHandler);
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
