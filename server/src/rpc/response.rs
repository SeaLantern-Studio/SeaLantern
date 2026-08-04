//! RPC 成功响应的传输无关包络。

use serde::Serialize;

use super::RpcRequestId;

/// 由统一调度器返回的成功响应。
///
/// `requestId` 与失败响应中的关联标识使用相同来源。适配器可以直接序列化该结构，或在
/// 保持旧兼容接口时仅取出 [`Self::into_data`]；RPC 方法本身不需要了解任何传输包络。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcResponse<T> {
    request_id: RpcRequestId,
    data: T,
}

impl<T> RpcResponse<T> {
    pub(crate) fn new(request_id: RpcRequestId, data: T) -> Self {
        Self { request_id, data }
    }

    /// 返回本次调用的关联标识。
    pub fn request_id(&self) -> &RpcRequestId {
        &self.request_id
    }

    /// 借用 RPC 方法返回的数据。
    pub fn data(&self) -> &T {
        &self.data
    }

    /// 取出 RPC 方法返回的数据，供兼容适配器使用。
    pub fn into_data(self) -> T {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn serializes_the_request_id_with_camel_case() {
        let request_id = RpcRequestId::new("response-42").expect("request id should be valid");
        let response = RpcResponse::new(request_id, json!({ "serverName": "Lantern" }));
        let value = serde_json::to_value(response).expect("response should serialize");

        assert_eq!(value["requestId"], "response-42");
        assert_eq!(value["data"]["serverName"], "Lantern");
        assert!(value.get("request_id").is_none());
    }
}
