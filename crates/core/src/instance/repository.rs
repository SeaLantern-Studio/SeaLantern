use super::{Instance, InstanceId};

/// 实例注册表的领域端口。
///
/// 持久化格式、锁和迁移均由 infra 或上层应用服务实现。
pub trait InstanceRepository {
    type Error: std::error::Error + Send + Sync + 'static;

    fn list(&self) -> Result<Vec<Instance>, Self::Error>;

    fn find(&self, id: &InstanceId) -> Result<Option<Instance>, Self::Error>;

    fn save(&self, instance: &Instance) -> Result<(), Self::Error>;

    fn remove(&self, id: &InstanceId) -> Result<bool, Self::Error>;
}
