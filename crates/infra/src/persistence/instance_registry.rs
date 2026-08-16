use std::fmt;

use rusqlite::params;

use super::{PersistenceError, SqlValue, SqliteDatabase};

const CREATE_INSTANCE_REGISTRY_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS instance_registry (
        instance_id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(instance_id)) > 0),
        payload TEXT NOT NULL,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
";

/// 不依赖领域模型的实例注册表记录。
///
/// `payload` 由调用方负责序列化和反序列化，避免基础设施 crate 依赖 `core`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceRegistryRecord {
    id: String,
    payload: String,
}

impl InstanceRegistryRecord {
    /// 使用非空实例标识和不透明持久化载荷构造记录。
    pub fn new(
        id: impl Into<String>,
        payload: impl Into<String>,
    ) -> Result<Self, InstanceRegistryError> {
        let id = normalize_id(id.into())?;
        Ok(Self { id, payload: payload.into() })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }
}

/// 实例注册表的 SQLite 持久化适配器。
///
/// 此类型只保存 ID 和调用方提供的载荷，不实现 `core::instance::InstanceRepository`。
#[derive(Clone)]
pub struct InstanceRegistry {
    database: SqliteDatabase,
}

impl InstanceRegistry {
    /// 初始化注册表表结构并绑定到已打开的数据库。
    pub async fn initialize(database: SqliteDatabase) -> Result<Self, InstanceRegistryError> {
        database
            .write("initialize instance registry", |transaction| {
                transaction.execute_batch(CREATE_INSTANCE_REGISTRY_TABLE)?;
                Ok(())
            })
            .await
            .map_err(InstanceRegistryError::Persistence)?;
        Ok(Self { database })
    }

    /// 按稳定 ID 排序列出所有注册表记录。
    pub async fn list(&self) -> Result<Vec<InstanceRegistryRecord>, InstanceRegistryError> {
        let rows = self
            .database
            .query_with_operation(
                "list instance registry records",
                "SELECT instance_id, payload FROM instance_registry ORDER BY instance_id",
                Vec::new(),
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .await
            .map_err(InstanceRegistryError::Persistence)?;
        rows.into_iter()
            .map(|(id, payload)| InstanceRegistryRecord::new(id, payload))
            .collect()
    }

    /// 查找一个实例注册表记录。
    pub async fn find(
        &self,
        id: &str,
    ) -> Result<Option<InstanceRegistryRecord>, InstanceRegistryError> {
        let id = normalize_id(id.to_owned())?;
        let mut rows = self
            .database
            .query_with_operation(
                "find instance registry record",
                "SELECT instance_id, payload FROM instance_registry WHERE instance_id = ?1",
                vec![SqlValue::Text(id)],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .await
            .map_err(InstanceRegistryError::Persistence)?;
        rows.pop()
            .map(|(id, payload)| InstanceRegistryRecord::new(id, payload))
            .transpose()
    }

    /// 以 ID 为键原子创建或更新一个记录。
    pub async fn save(&self, record: InstanceRegistryRecord) -> Result<(), InstanceRegistryError> {
        self.database
            .write("save instance registry record", move |transaction| {
                transaction.execute(
                    "INSERT INTO instance_registry (instance_id, payload) VALUES (?1, ?2) \
                     ON CONFLICT(instance_id) DO UPDATE SET \
                         payload = excluded.payload, \
                         updated_at = CURRENT_TIMESTAMP",
                    params![record.id, record.payload],
                )?;
                Ok(())
            })
            .await
            .map_err(InstanceRegistryError::Persistence)
    }

    /// 删除一个记录，并报告该 ID 是否原本存在。
    pub async fn remove(&self, id: &str) -> Result<bool, InstanceRegistryError> {
        let id = normalize_id(id.to_owned())?;
        self.database
            .write("remove instance registry record", move |transaction| {
                let affected = transaction
                    .execute("DELETE FROM instance_registry WHERE instance_id = ?1", params![id])?;
                Ok(affected != 0)
            })
            .await
            .map_err(InstanceRegistryError::Persistence)
    }
}

/// 实例注册表操作的可诊断错误。
#[derive(Debug)]
pub enum InstanceRegistryError {
    EmptyId,
    Persistence(PersistenceError),
}

impl fmt::Display for InstanceRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => write!(formatter, "instance registry ID cannot be empty"),
            Self::Persistence(error) => {
                write!(formatter, "instance registry operation failed: {error}")
            }
        }
    }
}

impl std::error::Error for InstanceRegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Persistence(error) => Some(error),
            Self::EmptyId => None,
        }
    }
}

fn normalize_id(id: String) -> Result<String, InstanceRegistryError> {
    let id = id.trim().to_owned();
    if id.is_empty() {
        return Err(InstanceRegistryError::EmptyId);
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{InstanceRegistry, InstanceRegistryError, InstanceRegistryRecord};
    use crate::persistence::SqliteDatabase;

    fn database_path(label: &str) -> PathBuf {
        crate::fs::test_dir(label).join("state.sqlite")
    }

    #[tokio::test]
    async fn saves_lists_finds_and_removes_records() {
        let path = database_path("instance-registry-crud");
        let registry = InstanceRegistry::initialize(SqliteDatabase::open(&path).await.unwrap())
            .await
            .unwrap();
        registry
            .save(InstanceRegistryRecord::new("beta", "first").unwrap())
            .await
            .unwrap();
        registry
            .save(InstanceRegistryRecord::new("alpha", "initial").unwrap())
            .await
            .unwrap();
        registry
            .save(InstanceRegistryRecord::new("alpha", "updated").unwrap())
            .await
            .unwrap();

        let records = registry.list().await.unwrap();
        assert_eq!(
            records,
            vec![
                InstanceRegistryRecord::new("alpha", "updated").unwrap(),
                InstanceRegistryRecord::new("beta", "first").unwrap(),
            ]
        );
        assert_eq!(
            registry.find(" alpha ").await.unwrap(),
            Some(InstanceRegistryRecord::new("alpha", "updated").unwrap())
        );
        assert!(registry.remove("alpha").await.unwrap());
        assert!(!registry.remove("alpha").await.unwrap());
        assert!(registry.find("alpha").await.unwrap().is_none());

        drop(registry);
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn rejects_empty_registry_ids() {
        assert!(matches!(
            InstanceRegistryRecord::new("  ", "payload"),
            Err(InstanceRegistryError::EmptyId)
        ));
    }
}
