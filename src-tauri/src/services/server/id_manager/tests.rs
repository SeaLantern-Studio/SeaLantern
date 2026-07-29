use super::*;

#[tokio::test]
async fn test_create_and_resolve_id() -> Result<(), String> {
    let manager = ServerIdManager::new();

    let req = CreateServerIdRequest {
        id: Some("test-server".to_string()),
        name: "Test Server".to_string(),
        address: "127.0.0.1".to_string(),
        port: 25565,
        description: Some("A test server".to_string()),
        tags: Some(vec!["test".to_string()]),
    };

    let entry = manager.create_id(req).await?;
    assert_eq!(entry.id, "test-server");
    assert_eq!(entry.name, "Test Server");

    let (addr, port) = manager.resolve_id("test-server").await?;
    assert_eq!(addr, "127.0.0.1");
    assert_eq!(port, 25565);

    Ok(())
}

#[tokio::test]
async fn test_duplicate_id() -> Result<(), String> {
    let manager = ServerIdManager::new();

    let req1 = CreateServerIdRequest {
        id: Some("duplicate".to_string()),
        name: "Server 1".to_string(),
        address: "127.0.0.1".to_string(),
        port: 25565,
        description: None,
        tags: None,
    };

    manager.create_id(req1).await?;

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

    Ok(())
}

#[tokio::test]
async fn test_search_ids() -> Result<(), String> {
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

    manager.create_id(req1).await?;
    manager.create_id(req2).await?;

    let results = manager.search_ids("survival").await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "survival-1");

    Ok(())
}
