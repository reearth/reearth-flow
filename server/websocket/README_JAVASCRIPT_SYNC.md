# JavaScript 风格 WebSocket 服务器运行指南

本文档介绍如何运行升级后的 Rust WebSocket 服务器，现在完全兼容 JavaScript 版本的 Redis 同步逻辑。

## 🚀 快速启动

### 1. 环境要求

- Rust 1.70+
- Redis 服务器 (支持 Redis Streams)
- Google Cloud Storage (可选，用于持久化存储)

### 2. 配置环境变量

```bash
# Redis 配置
export REDIS_URL="redis://localhost:6379"

# GCS 配置 (可选)
export GOOGLE_CLOUD_STORAGE_BUCKET="your-bucket-name"

# WebSocket 端口
export WS_PORT="3001"

# 日志级别
export RUST_LOG=info
```

### 3. 运行服务器

```bash
# 切换到 WebSocket 目录
cd /Users/xy/work/eukarya/reearth-flow/server/websocket

# 运行服务器
cargo run --bin websocket

# 或者构建后运行
cargo build --release --bin websocket
./target/release/websocket
```

### 4. 带认证功能运行 (可选)

```bash
cargo run --bin websocket --features auth
```

## 🔧 新功能特性

### JavaScript 兼容的 Redis 同步

✅ **消息格式统一**: 使用 `m` 字段存储消息内容  
✅ **Stream 命名**: `{prefix}:room:{room}:{docid}` 格式  
✅ **智能消息合并**: 减少网络传输开销  
✅ **自动文档压缩**: Worker 定期压缩和清理 Redis 流  

### 实时协作功能

✅ **多客户端同步**: 支持无限客户端实时编辑  
✅ **Awareness 同步**: 实时显示用户光标和选择  
✅ **断线重连**: 自动恢复连接状态  
✅ **冲突解决**: Y.js CRDT 算法确保数据一致性  

### 企业级特性

✅ **高性能**: 优化的 Redis 订阅机制  
✅ **可扩展**: 多实例部署支持  
✅ **持久化**: GCS 自动备份文档状态  
✅ **监控**: 详细的日志和指标  

## 📊 系统架构

```
Client <-> WebSocket Server <-> Redis Streams <-> GCS Storage
                   |
                   v
            JavaScript-compatible
            API/Subscriber/Worker
```

### 核心组件

1. **Api**: JavaScript 风格的文档和消息处理
2. **Subscriber**: Redis 流订阅和消息分发
3. **Worker**: 自动文档压缩和清理
4. **Protocol**: 消息编码/解码和智能合并
5. **BroadcastGroup**: 实时消息广播

## 🔍 故障排除

### 客户端断开连接警告

如果看到类似警告：
```
WARN failed to send awareness update: channel closed
```

这是**正常现象**，表示客户端断开连接时的资源清理。我们已经优化了这些警告：
- ✅ 添加了通道状态检查
- ✅ 实现了优雅的资源清理  
- ✅ 定期清理不活跃的连接组

### 清理 Redis 历史数据

如果遇到消息解析错误，请清理 Redis：

```bash
# 方式1: 使用提供的清理脚本
./scripts/clean_redis.sh

# 方式2: 手动清理所有数据
redis-cli FLUSHALL

# 方式3: 只清理特定前缀
redis-cli --scan --pattern "y:*" | xargs redis-cli DEL
```

### Redis 连接问题

```bash
# 检查 Redis 是否运行
redis-cli ping

# 检查 Redis Streams 支持
redis-server --version  # 需要 5.0+
```

### 端口占用

```bash
# 检查端口是否被占用
netstat -an | grep :3001

# 使用不同端口
export WS_PORT="3002"
```

### 日志调试

```bash
# 启用详细日志
export RUST_LOG=debug
cargo run --bin websocket

# 只显示 WebSocket 相关日志
export RUST_LOG=websocket=debug
```

## 🧪 测试兼容性

### 与 JavaScript 版本测试

1. 启动 JavaScript WebSocket 服务器
2. 启动 Rust WebSocket 服务器 (不同端口)
3. 连接客户端到两个服务器
4. 验证消息在两个服务器间正确同步

### 压力测试

```bash
# 使用 wscat 测试连接
npm install -g wscat
wscat -c ws://localhost:3001

# 批量测试工具
cargo run --example stress_test
```

## 📈 性能监控

### Redis 监控

```bash
# 监控 Redis 流
redis-cli MONITOR

# 检查流信息
redis-cli XINFO STREAM y:room:test:index
```

### 系统监控

```bash
# CPU 和内存使用
htop

# 网络连接
ss -tuln | grep :3001
```

## 🔄 优雅关闭

服务器支持优雅关闭，按 `Ctrl+C` 时会：

1. 停止接收新连接
2. 完成正在处理的请求
3. 关闭 Worker 队列
4. 销毁 Subscriber 连接
5. 清理 API 资源
6. 关闭服务器

## 📝 配置选项

服务器配置通过环境变量或配置文件设置：

```toml
[redis]
url = "redis://localhost:6379"
prefix = "y"
task_debounce = 10000
min_message_lifetime = 60000

[websocket]
port = "3001"
buffer_capacity = 512

[storage]
bucket = "your-gcs-bucket"
```

## 🤝 与 JavaScript 版本的兼容性

现在 Rust 和 JavaScript 版本可以：

✅ 共享相同的 Redis 基础设施  
✅ 处理相同的消息格式  
✅ 支持相同的 API 接口  
✅ 使用相同的文档存储格式  
✅ 实现相同的协作行为  

两个版本可以在同一个系统中并行运行，客户端可以连接到任一版本并保持完全兼容！🚀
