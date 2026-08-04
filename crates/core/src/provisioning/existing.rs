use crate::instance::{
    plan_import, InstanceImportError, InstanceImportPlan, InstanceImportRequest,
};

/// 为已有服务器目录构建导入计划。
///
/// 该函数只处理路径与启动目标合同；目录扫描和文件操作由上层实现。
pub fn plan_existing_instance(
    request: InstanceImportRequest,
) -> Result<InstanceImportPlan, ExistingInstanceError> {
    plan_import(request).map_err(ExistingInstanceError::Import)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExistingInstanceError {
    Import(InstanceImportError),
}

impl std::fmt::Display for ExistingInstanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Import(error) => write!(formatter, "invalid existing instance import: {error}"),
        }
    }
}

impl std::error::Error for ExistingInstanceError {}
