//! RPC 方法调用契约和统一调度器。

use std::future::Future;

use crate::observability;

use super::{RpcContext, RpcMethodName, RpcRequest, RpcResult};

/// 一个传输无关的 RPC 应用服务方法。
///
/// 方法借用服务实例、消费已解析参数，并只返回 [`RpcResult`]。Tauri 与 Axum 适配器负责
/// 参数反序列化、授权、协议错误映射和事件推送；方法实现只编排领域端口。
pub trait RpcMethod {
    /// 稳定的方法名，供日志和适配器注册表使用。
    const NAME: RpcMethodName;

    /// 已完成传输反序列化的输入类型。
    type Request: Send;
    /// 可由传输适配器序列化的输出类型。
    type Response: Send;

    /// 执行方法。
    fn call(
        &self,
        context: &RpcContext,
        request: Self::Request,
    ) -> impl Future<Output = RpcResult<Self::Response>> + Send;
}

/// 调度一个 RPC 方法并为失败结果补充关联标识和结构化追踪事件。
///
/// 不记录请求参数、响应正文或错误消息，避免凭据、命令正文和文件路径进入公共事件流。
pub async fn dispatch<M>(method: &M, request: RpcRequest<M::Request>) -> RpcResult<M::Response>
where
    M: RpcMethod + Sync,
{
    let (context, params) = request.into_parts();
    match method.call(&context, params).await {
        Ok(response) => Ok(response),
        Err(error) => {
            let error = error.with_request_id(context.request_id().clone());
            observability::rpc_method_failed(M::NAME.as_str(), &context, &error);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::{RpcError, RpcErrorCode, RpcRequestId, RpcTransport};

    struct EchoMethod;

    impl RpcMethod for EchoMethod {
        const NAME: RpcMethodName = RpcMethodName::new("test.echo");

        type Request = String;
        type Response = String;

        fn call(
            &self,
            context: &RpcContext,
            request: Self::Request,
        ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
            let request_id = context.request_id().to_string();
            async move { Ok(format!("{request_id}:{request}")) }
        }
    }

    struct FailingMethod;

    impl RpcMethod for FailingMethod {
        const NAME: RpcMethodName = RpcMethodName::new("test.fail");

        type Request = ();
        type Response = ();

        fn call(
            &self,
            _context: &RpcContext,
            _request: Self::Request,
        ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
            async { Err(RpcError::unavailable("test dependency")) }
        }
    }

    fn request<T>(params: T) -> RpcRequest<T> {
        let request_id = RpcRequestId::new("rpc-test-42").expect("request id should be valid");
        let context = RpcContext::new(request_id, RpcTransport::Internal);
        RpcRequest::new(context, params)
    }

    #[tokio::test]
    async fn dispatch_passes_owned_request_data_to_the_method() {
        let response = dispatch(&EchoMethod, request("payload".to_string()))
            .await
            .expect("method should succeed");

        assert_eq!(response, "rpc-test-42:payload");
    }

    #[tokio::test]
    async fn dispatch_attaches_the_request_id_to_safe_failures() {
        let error = dispatch(&FailingMethod, request(()))
            .await
            .expect_err("method should fail");

        assert_eq!(error.code(), RpcErrorCode::Unavailable);
        assert_eq!(error.request_id().map(RpcRequestId::as_str), Some("rpc-test-42"));
        assert!(error.is_retryable());
    }
}
