use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 从环境变量获取Redis URL
    let redis_url = std::env::var("REEARTH_FLOW_REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    println!("Testing Redis connection to: {}", redis_url);

    // 测试使用redis crate直接连接
    match redis::Client::open(redis_url.clone()) {
        Ok(client) => {
            println!("Redis client created successfully");

            match client.get_multiplexed_async_connection().await {
                Ok(mut conn) => {
                    println!("✓ Successfully connected to Redis!");

                    // 尝试执行一个简单的PING命令
                    let pong: Result<String, redis::RedisError> =
                        redis::cmd("PING").query_async(&mut conn).await;

                    match pong {
                        Ok(response) => println!("✓ PING response: {}", response),
                        Err(e) => println!("✗ PING failed: {}", e),
                    }
                }
                Err(e) => {
                    println!("✗ Failed to connect to Redis: {}", e);
                    println!("Error details: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to create Redis client: {}", e);
        }
    }

    Ok(())
}
