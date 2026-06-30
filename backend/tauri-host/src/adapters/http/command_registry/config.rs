use super::common::{parse_params, CommandHandler, RegistryBuilder};
use super::requests::{
    ListServerConfigFilesRequest, ParseServerPropertiesSourceRequest,
    PreviewServerPropertiesWriteFromSourceRequest, PreviewServerPropertiesWriteRequest,
    ReadConfigRequest, ReadServerConfigDocumentRequest, ReadServerConfigSourceRequest,
    ReadServerPropertiesRequest, SearchServerConfigFilesRequest, WriteConfigRequest,
    WriteServerConfigDocumentRequest, WriteServerConfigSourceRequest, WriteServerPropertiesRequest,
    WriteServerPropertiesSourceRequest,
};
use crate::commands::server::config as config_commands;
use serde_json::Value;
pub(super) fn register_handlers(builder: &mut RegistryBuilder) {
    builder.register("read_config", handle_read_config as CommandHandler);
    builder.register("list_server_config_files", handle_list_server_config_files as CommandHandler);
    builder.register(
        "search_server_config_files",
        handle_search_server_config_files as CommandHandler,
    );
    builder
        .register("read_server_config_source", handle_read_server_config_source as CommandHandler);
    builder.register(
        "write_server_config_source",
        handle_write_server_config_source as CommandHandler,
    );
    builder.register(
        "read_server_config_document",
        handle_read_server_config_document as CommandHandler,
    );
    builder.register(
        "write_server_config_document",
        handle_write_server_config_document as CommandHandler,
    );
    builder.register("write_config", handle_write_config as CommandHandler);
    builder.register("read_server_properties", handle_read_server_properties as CommandHandler);
    builder.register("write_server_properties", handle_write_server_properties as CommandHandler);
    builder.register(
        "read_server_properties_source",
        handle_read_server_properties_source as CommandHandler,
    );
    builder.register(
        "write_server_properties_source",
        handle_write_server_properties_source as CommandHandler,
    );
    builder.register(
        "parse_server_properties_source",
        handle_parse_server_properties_source as CommandHandler,
    );
    builder.register(
        "preview_server_properties_write",
        handle_preview_server_properties_write as CommandHandler,
    );
    builder.register(
        "preview_server_properties_write_from_source",
        handle_preview_server_properties_write_from_source as CommandHandler,
    );
}

fn handle_read_server_config_source(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ReadServerConfigSourceRequest = parse_params(params)?;
        let result =
            config_commands::read_server_config_source(
                req.server_path,
                req.relative_path,
                req.locator,
                req.discovery_options,
            )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_write_server_config_source(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: WriteServerConfigSourceRequest = parse_params(params)?;
        config_commands::write_server_config_source(
            req.server_path,
            req.relative_path,
            req.locator,
            req.discovery_options,
            req.source,
        )?;
        Ok(Value::Null)
    })
}

fn handle_read_server_config_document(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ReadServerConfigDocumentRequest = parse_params(params)?;
        let result =
            config_commands::read_server_config_document(
                req.server_path,
                req.relative_path,
                req.locator,
                req.discovery_options,
            )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_write_server_config_document(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: WriteServerConfigDocumentRequest = parse_params(params)?;
        config_commands::write_server_config_document(
            req.server_path,
            req.relative_path,
            req.locator,
            req.discovery_options,
            req.content,
        )?;
        Ok(Value::Null)
    })
}

fn handle_list_server_config_files(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ListServerConfigFilesRequest = parse_params(params)?;
        let result = config_commands::list_server_config_files(req.server_path, req.discovery_options)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_search_server_config_files(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: SearchServerConfigFilesRequest = parse_params(params)?;
        let result = config_commands::search_server_config_files(
            req.server_path,
            req.query,
            req.mode,
            req.scope,
            req.discovery_options,
            req.limit,
            req.case_sensitive,
        )?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_read_config(params: Value) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ReadConfigRequest = parse_params(params)?;
        let result = config_commands::read_config(req.server_path, req.path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_write_config(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: WriteConfigRequest = parse_params(params)?;
        config_commands::write_config(req.server_path, req.path, req.values)?;
        Ok(Value::Null)
    })
}

fn handle_read_server_properties(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ReadServerPropertiesRequest = parse_params(params)?;
        let result = config_commands::read_server_properties(req.server_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_write_server_properties(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: WriteServerPropertiesRequest = parse_params(params)?;
        config_commands::write_server_properties(req.server_path, req.values)?;
        Ok(Value::Null)
    })
}

fn handle_read_server_properties_source(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ReadServerPropertiesRequest = parse_params(params)?;
        let result = config_commands::read_server_properties_source(req.server_path)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_write_server_properties_source(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: WriteServerPropertiesSourceRequest = parse_params(params)?;
        config_commands::write_server_properties_source(req.server_path, req.source)?;
        Ok(Value::Null)
    })
}

fn handle_parse_server_properties_source(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: ParseServerPropertiesSourceRequest = parse_params(params)?;
        let result = config_commands::parse_server_properties_source(req.source)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_preview_server_properties_write(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PreviewServerPropertiesWriteRequest = parse_params(params)?;
        let result = config_commands::preview_server_properties_write(req.server_path, req.values)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}

fn handle_preview_server_properties_write_from_source(
    params: Value,
) -> futures::future::BoxFuture<'static, Result<Value, String>> {
    Box::pin(async move {
        let req: PreviewServerPropertiesWriteFromSourceRequest = parse_params(params)?;
        let result =
            config_commands::preview_server_properties_write_from_source(req.source, req.values)?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    })
}
