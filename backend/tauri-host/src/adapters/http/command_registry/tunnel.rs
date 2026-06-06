use super::common::{parse_params, CommandHandler, RegistryBuilder};
use super::requests::{TunnelHostRequest, TunnelJoinRequest};
use crate::commands::online::tunnel as tunnel_commands;
use serde_json::Value;
pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("tunnel_host", handle_tunnel_host as CommandHandler);
    builder.register("tunnel_join", handle_tunnel_join as CommandHandler);
    builder.register("tunnel_stop", handle_tunnel_stop as CommandHandler);
    builder.register("tunnel_status", handle_tunnel_status as CommandHandler);
    builder.register("tunnel_copy_ticket", handle_tunnel_copy_ticket as CommandHandler);
    builder.register("tunnel_regenerate_ticket", handle_tunnel_regenerate_ticket as CommandHandler);
    builder.register("tunnel_generate_ticket", handle_tunnel_generate_ticket as CommandHandler);
}

fn handle_tunnel_host(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: TunnelHostRequest = parse_params(params)?;
        let result =
            tunnel_commands::tunnel_host(req.port, req.password, req.max_players, req.relay_url)
                .await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_tunnel_join(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: TunnelJoinRequest = parse_params(params)?;
        let result = tunnel_commands::tunnel_join(req.ticket, req.local_port, req.password).await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_tunnel_stop(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = tunnel_commands::tunnel_stop().await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_tunnel_status(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = tunnel_commands::tunnel_status().await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_tunnel_copy_ticket(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let copied = tunnel_commands::tunnel_copy_ticket().await?;
        serde_json::to_value(copied).map_err(|e| e.to_string())
    })
}

fn handle_tunnel_regenerate_ticket(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = tunnel_commands::tunnel_regenerate_ticket().await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_tunnel_generate_ticket(
    _params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let result = tunnel_commands::tunnel_generate_ticket().await?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}
