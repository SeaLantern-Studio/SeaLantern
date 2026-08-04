use crate::instance::{Instance, InstanceError, InstanceSpec};

/// 新建空实例的无副作用计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateInstancePlan {
    pub instance: Instance,
}

/// 校验创建输入并生成待持久化实例。
pub fn plan_create(instance: InstanceSpec) -> Result<CreateInstancePlan, CreateInstanceError> {
    tracing::debug!(
        target: "sealantern.core.provisioning.create",
        instance_id = %instance.id.as_str(),
        directory = %instance.directory.display(),
        "planning instance creation"
    );
    let instance = Instance::new(instance).map_err(CreateInstanceError::Instance)?;
    tracing::debug!(
        target: "sealantern.core.provisioning.create",
        instance_id = %instance.id.as_str(),
        "instance creation plan ready"
    );
    Ok(CreateInstancePlan { instance })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateInstanceError {
    Instance(InstanceError),
}

impl std::fmt::Display for CreateInstanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Instance(error) => {
                write!(formatter, "invalid instance creation request: {error}")
            }
        }
    }
}

impl std::error::Error for CreateInstanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Instance(error) => Some(error),
        }
    }
}
