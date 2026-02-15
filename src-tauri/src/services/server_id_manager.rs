use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use uuid::Uuid;

/// Server ID entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerIdEntry {
    /// Unique server ID (e.g., "manus-survival-01")
    pub id: String,
    /// Human-readable server name
    pub name: String,
    /// Server address (IP or hostname)
    pub address: String,
    /// Server port
    pub port: u16,
    /// Server description
    pub description: Option<String>,
    /// When this ID was created (Unix timestamp)
    pub created_at: u64,
    /// When this ID was last accessed
    pub last_accessed_at: Option<u64>,
    /// Whether this ID is active/public
    pub is_active: bool,
    /// Optional tags for categorization
    pub tags: Vec<String>,
}

/// Request to create a new server ID
#[derive(Debug, Deserialize)]
pub struct CreateServerIdRequest {
    pub id: Option<String>, // If None, auto-generate
    pub name: String,
    pub address: String,
    pub port: u16,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Response for server ID operations
#[derive(Debug, Serialize)]
pub struct ServerIdResponse {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub created_at: u64,
    pub is_active: bool,
}

/// Server ID Manager - handles generation, storage, and lookup
pub struct ServerIdManager {
    // In-memory storage (in production, use database)
    entries: Arc<RwLock<HashMap<String, ServerIdEntry>>>,
}

impl ServerIdManager {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a random server ID if not provided
    fn generate_id() -> String {
        format!("srv-{}", Uuid::new_v4().to_string()[..8].to_lowercase())
    }

    /// Generate a human-readable ID from server name
    fn generate_id_from_name(name: &str) -> String {
        let sanitized = name
            .to_lowercase()
            .replace(" ", "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect::<String>();
        
        format!("{}-{}", sanitized, Uuid::new_v4().to_string()[..4].to_lowercase())
    }

    /// Create a new server ID
    pub async fn create_id(&self, req: CreateServerIdRequest) -> Result<ServerIdEntry, String> {
        let id = if let Some(custom_id) = req.id {
            // Validate custom ID format
            if custom_id.len() < 3 || custom_id.len() > 32 {
                return Err("Server ID must be between 3 and 32 characters".to_string());
            }
            if !custom_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return Err("Server ID can only contain alphanumeric characters, hyphens, and underscores".to_string());
            }
            custom_id
        } else {
            Self::generate_id_from_name(&req.name)
        };

        // Check if ID already exists
        let entries = self.entries.read().await;
        if entries.contains_key(&id) {
            return Err(format!("Server ID '{}' already exists", id));
        }
        drop(entries);

        let now = Utc::now().timestamp() as u64;
        let entry = ServerIdEntry {
            id: id.clone(),
            name: req.name,
            address: req.address,
            port: req.port,
            description: req.description,
            created_at: now,
            last_accessed_at: None,
            is_active: true,
            tags: req.tags.unwrap_or_default(),
        };

        let mut entries = self.entries.write().await;
        entries.insert(id.clone(), entry.clone());

        Ok(entry)
    }

    /// Resolve a server ID to its address
    pub async fn resolve_id(&self, id: &str) -> Result<(String, u16), String> {
        let mut entries = self.entries.write().await;
        
        match entries.get_mut(id) {
            Some(entry) => {
                if !entry.is_active {
                    return Err(format!("Server ID '{}' is inactive", id));
                }
                // Update last accessed time
                entry.last_accessed_at = Some(Utc::now().timestamp() as u64);
                Ok((entry.address.clone(), entry.port))
            }
            None => Err(format!("Server ID '{}' not found", id)),
        }
    }

    /// Get server ID details
    pub async fn get_id(&self, id: &str) -> Result<ServerIdEntry, String> {
        let entries = self.entries.read().await;
        entries
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Server ID '{}' not found", id))
    }

    /// List all active server IDs
    pub async fn list_ids(&self) -> Vec<ServerIdEntry> {
        let entries = self.entries.read().await;
        entries
            .values()
            .filter(|e| e.is_active)
            .cloned()
            .collect()
    }

    /// Update a server ID
    pub async fn update_id(&self, id: &str, name: Option<String>, address: Option<String>, port: Option<u16>) -> Result<ServerIdEntry, String> {
        let mut entries = self.entries.write().await;
        
        match entries.get_mut(id) {
            Some(entry) => {
                if let Some(n) = name {
                    entry.name = n;
                }
                if let Some(a) = address {
                    entry.address = a;
                }
                if let Some(p) = port {
                    entry.port = p;
                }
                Ok(entry.clone())
            }
            None => Err(format!("Server ID '{}' not found", id)),
        }
    }

    /// Deactivate a server ID
    pub async fn deactivate_id(&self, id: &str) -> Result<(), String> {
        let mut entries = self.entries.write().await;
        
        match entries.get_mut(id) {
            Some(entry) => {
                entry.is_active = false;
                Ok(())
            }
            None => Err(format!("Server ID '{}' not found", id)),
        }
    }

    /// Delete a server ID
    pub async fn delete_id(&self, id: &str) -> Result<(), String> {
        let mut entries = self.entries.write().await;
        
        if entries.remove(id).is_some() {
            Ok(())
        } else {
            Err(format!("Server ID '{}' not found", id))
        }
    }

    /// Search server IDs by name or tags
    pub async fn search_ids(&self, query: &str) -> Vec<ServerIdEntry> {
        let entries = self.entries.read().await;
        let query_lower = query.to_lowercase();
        
        entries
            .values()
            .filter(|e| {
                e.is_active && (
                    e.name.to_lowercase().contains(&query_lower)
                    || e.id.to_lowercase().contains(&query_lower)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                )
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_resolve_id() {
        let manager = ServerIdManager::new();
        
        let req = CreateServerIdRequest {
            id: Some("test-server".to_string()),
            name: "Test Server".to_string(),
            address: "127.0.0.1".to_string(),
            port: 25565,
            description: Some("A test server".to_string()),
            tags: Some(vec!["test".to_string()]),
        };

        let entry = manager.create_id(req).await.unwrap();
        assert_eq!(entry.id, "test-server");
        assert_eq!(entry.name, "Test Server");

        let (addr, port) = manager.resolve_id("test-server").await.unwrap();
        assert_eq!(addr, "127.0.0.1");
        assert_eq!(port, 25565);
    }

    #[tokio::test]
    async fn test_duplicate_id() {
        let manager = ServerIdManager::new();
        
        let req1 = CreateServerIdRequest {
            id: Some("duplicate".to_string()),
            name: "Server 1".to_string(),
            address: "127.0.0.1".to_string(),
            port: 25565,
            description: None,
            tags: None,
        };

        manager.create_id(req1).await.unwrap();

        let req2 = CreateServerIdRequest {
            id: Some("duplicate".to_string()),
            name: "Server 2".to_string(),
            address: "127.0.0.2".to_string(),
            port: 25566,
            description: None,
            tags: None,
        };

        let result = manager.create_id(req2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_ids() {
        let manager = ServerIdManager::new();
        
        let req1 = CreateServerIdRequest {
            id: Some("survival-1".to_string()),
            name: "Survival World".to_string(),
            address: "127.0.0.1".to_string(),
            port: 25565,
            description: None,
            tags: Some(vec!["survival".to_string()]),
        };

        let req2 = CreateServerIdRequest {
            id: Some("creative-1".to_string()),
            name: "Creative World".to_string(),
            address: "127.0.0.2".to_string(),
            port: 25566,
            description: None,
            tags: Some(vec!["creative".to_string()]),
        };

        manager.create_id(req1).await.unwrap();
        manager.create_id(req2).await.unwrap();

        let results = manager.search_ids("survival").await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "survival-1");
    }
}
