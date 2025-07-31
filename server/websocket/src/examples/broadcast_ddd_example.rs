/// Example demonstrating how to use the DDD-refactored broadcast system
/// 
/// This example shows how to:
/// 1. Set up the DDD architecture with proper dependency injection
/// 2. Create broadcast groups using the new service layer
/// 3. Handle WebSocket connections with Y.js support
/// 4. Manage document collaboration and awareness

use crate::broadcast::{BroadcastGroupDDD, BroadcastGroupDDDFactory};
use crate::domain::entity::broadcast::BroadcastConfig;
use crate::domain::value_objects::document_name::DocumentName;
use crate::domain::value_objects::instance_id::InstanceId;
use crate::infrastructure::persistence::gcs::GcsStore;
use crate::infrastructure::redis::RedisStore;
use anyhow::Result;
use std::sync::Arc;

/// Example of setting up and using the DDD broadcast system
pub async fn run_broadcast_ddd_example() -> Result<()> {
    println!("🚀 Starting DDD Broadcast Example");

    // 1. Set up infrastructure dependencies
    let gcs_store = Arc::new(GcsStore::new("reearth-flow-docs".to_string()));
    let redis_store = Arc::new(RedisStore::new("redis://localhost:6379").await?);

    // 2. Configure broadcast behavior
    let config = BroadcastConfig {
        buffer_capacity: 1024,
        heartbeat_interval_ms: 30000,
        sync_interval_ms: 5000,
        awareness_update_interval_ms: 1000,
    };

    // 3. Create factory for broadcast groups
    let factory = BroadcastGroupDDDFactory::new(
        gcs_store.clone(),
        redis_store.clone(),
        config,
    );

    // 4. Create a document and instance
    let document_name = DocumentName::new("collaborative-doc-1".to_string())?;
    let instance_id = InstanceId::new();

    println!("📄 Document: {}", document_name.as_str());
    println!("🆔 Instance: {}", instance_id.as_str());

    // 5. Create DDD broadcast group
    let broadcast_group = factory.create_group(document_name.clone(), instance_id).await?;

    println!("✅ Created DDD broadcast group");

    // 6. Demonstrate basic operations
    demonstrate_basic_operations(&broadcast_group).await?;

    // 7. Demonstrate message broadcasting
    demonstrate_message_broadcasting(&broadcast_group).await?;

    // 8. Demonstrate document persistence
    demonstrate_document_persistence(&broadcast_group).await?;

    // 9. Start background tasks
    broadcast_group.start_background_tasks().await?;
    println!("🔄 Started background tasks (awareness updates, Redis sync, heartbeat)");

    // 10. Cleanup
    broadcast_group.shutdown().await?;
    println!("🛑 Shutdown complete");

    Ok(())
}

async fn demonstrate_basic_operations(broadcast_group: &BroadcastGroupDDD) -> Result<()> {
    println!("\n📊 Demonstrating basic operations:");

    // Get initial connection count
    let count = broadcast_group.connection_count().await?;
    println!("   Initial connections: {}", count);

    // Simulate connections
    let count = broadcast_group.increment_connections().await?;
    println!("   After increment: {}", count);

    let count = broadcast_group.increment_connections().await?;
    println!("   After second increment: {}", count);

    let count = broadcast_group.decrement_connections().await?;
    println!("   After decrement: {}", count);

    Ok(())
}

async fn demonstrate_message_broadcasting(broadcast_group: &BroadcastGroupDDD) -> Result<()> {
    println!("\n📡 Demonstrating message broadcasting:");

    // Subscribe to updates
    let mut receiver = broadcast_group.subscribe_to_updates().await?;
    println!("   Subscribed to document updates");

    // Broadcast a message in a separate task
    let broadcast_group_clone = broadcast_group.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let message = bytes::Bytes::from("Hello from DDD broadcast!");
        let _ = broadcast_group_clone.broadcast_message(message).await;
    });

    // Try to receive the message (with timeout)
    match tokio::time::timeout(
        tokio::time::Duration::from_secs(1),
        receiver.recv()
    ).await {
        Ok(Ok(message)) => {
            println!("   Received message: {:?}", String::from_utf8_lossy(&message));
        }
        Ok(Err(_)) => {
            println!("   No message received (channel closed)");
        }
        Err(_) => {
            println!("   Timeout waiting for message");
        }
    }

    Ok(())
}

async fn demonstrate_document_persistence(broadcast_group: &BroadcastGroupDDD) -> Result<()> {
    println!("\n💾 Demonstrating document persistence:");

    // Create some sample document data
    let sample_data = b"Sample Y.js document state";
    
    // Save snapshot
    broadcast_group.save_snapshot(sample_data).await?;
    println!("   Saved document snapshot ({} bytes)", sample_data.len());

    // Load document
    match broadcast_group.load_document().await? {
        Some(data) => {
            println!("   Loaded document ({} bytes)", data.len());
            println!("   Content preview: {:?}", String::from_utf8_lossy(&data[..20.min(data.len())]));
        }
        None => {
            println!("   No document found in storage");
        }
    }

    Ok(())
}

/// Example of handling WebSocket connections with the DDD architecture
pub async fn handle_websocket_with_ddd(
    websocket_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>
    >,
    document_name: String,
    user_token: Option<String>,
) -> Result<()> {
    println!("🔌 Handling WebSocket connection with DDD architecture");

    // Set up infrastructure
    let gcs_store = Arc::new(GcsStore::new("reearth-flow-docs".to_string()));
    let redis_store = Arc::new(RedisStore::new("redis://localhost:6379").await?);
    let config = BroadcastConfig::default();

    // Create factory and broadcast group
    let factory = BroadcastGroupDDDFactory::new(gcs_store, redis_store, config);
    let document_name = DocumentName::new(document_name)?;
    let instance_id = InstanceId::new();
    
    let broadcast_group = factory.create_group(document_name, instance_id).await?;

    // Handle the WebSocket connection
    broadcast_group.handle_websocket_connection(websocket_stream, user_token).await?;

    println!("✅ WebSocket connection handled successfully");
    Ok(())
}

/// Comparison between old and new architecture
pub fn architecture_comparison() {
    println!("\n🏗️  Architecture Comparison:");
    println!("
OLD ARCHITECTURE (src/broadcast/group.rs):
├── Monolithic BroadcastGroup struct
├── Direct dependencies on storage and Redis
├── Mixed concerns (business logic + infrastructure)
├── Hard to test and mock
└── Tightly coupled components

NEW DDD ARCHITECTURE:
├── Domain Layer
│   ├── Entities: BroadcastGroup, BroadcastConfig
│   ├── Value Objects: DocumentName, InstanceId
│   └── Repository Interfaces: BroadcastRepository, AwarenessRepository
├── Application Layer
│   └── Services: BroadcastGroupService (orchestrates business logic)
├── Infrastructure Layer
│   └── Repository Implementations: BroadcastRepositoryImpl, etc.
└── Interface Layer
    └── WebSocket Handlers: BroadcastWebSocketHandler

BENEFITS:
✅ Clean separation of concerns
✅ Easy to test with dependency injection
✅ Maintainable and extensible
✅ Follows DDD principles
✅ Backward compatible through facade pattern
    ");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ddd_example() {
        // This test demonstrates the DDD architecture in action
        // Note: Requires Redis and GCS to be available for full testing
        
        // For unit testing, you would mock the repository implementations
        println!("DDD broadcast example test - check logs for demonstration");
        
        // In a real test, you would:
        // 1. Create mock implementations of the repositories
        // 2. Inject them into the service
        // 3. Test business logic without external dependencies
    }
}
