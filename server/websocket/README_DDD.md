# WebSocket Server - DDD架构

该项目已经使用领域驱动设计（DDD）架构进行了重构。

## 架构概览

项目采用了经典的DDD分层架构：

```
src/
├── domain/           # 领域层
│   ├── models/      # 实体和值对象
│   ├── services/    # 领域服务
│   └── repositories/ # 仓储接口
├── application/     # 应用层
│   ├── services/    # 应用服务
│   └── dto/         # 数据传输对象
├── infrastructure/  # 基础设施层
│   ├── storage/     # 存储实现（GCS、Redis）
│   ├── broadcast/   # 广播实现
│   ├── websocket/   # WebSocket实现
│   └── tools/       # 工具函数
├── interface/       # 接口层
│   ├── http/        # HTTP API
│   ├── ws/          # WebSocket路由
│   └── server.rs    # 服务器启动
└── main.rs          # 应用入口
```

## 各层职责

### 领域层 (Domain Layer)
- **Models**: 包含核心业务实体如 `Document`、`BroadcastMessage`、`ConnectionInfo`
- **Services**: 领域服务如 `DocumentService`、`BroadcastService`
- **Repositories**: 定义仓储接口，不包含具体实现

### 应用层 (Application Layer)
- **Services**: 应用服务协调领域服务，如 `WebSocketService`、`DocumentAppService`
- **DTO**: 数据传输对象如 `AppState`、`RollbackRequest`
- **Config**: 配置管理服务

### 基础设施层 (Infrastructure Layer)
- **Storage**: 实现具体的存储逻辑（GCS存储、Redis存储）
- **Broadcast**: WebSocket广播实现
- **WebSocket**: WebSocket连接管理
- **Tools**: 工具函数（如压缩/解压缩）

### 接口层 (Interface Layer)
- **HTTP**: RESTful API端点
- **WS**: WebSocket端点
- **Server**: 服务器启动和配置

## 主要改进

1. **删除了认证功能**: 移除了所有 `auth` 相关代码
2. **统一错误处理**: 使用 `anyhow::Result` 进行错误处理
3. **清晰的依赖方向**: 依赖只能从外层指向内层
4. **解耦业务逻辑**: 业务逻辑集中在领域层
5. **可测试性**: 通过依赖注入和接口抽象提高了可测试性

## 运行项目

```bash
# 设置环境变量（或创建.env文件）
export REEARTH_FLOW_REDIS_URL=redis://127.0.0.1:6379
export REEARTH_FLOW_GCS_BUCKET_NAME=your-bucket-name
export REEARTH_FLOW_WS_PORT=8000

# 构建项目
cargo build

# 运行项目
cargo run
```

## API端点

- WebSocket: `ws://localhost:8000/{doc_id}`
- HTTP API:
  - GET `/api/document/{doc_id}` - 获取文档
  - GET `/api/document/{doc_id}/history` - 获取历史
  - POST `/api/document/{doc_id}/rollback` - 回滚文档
  - POST `/api/document/{doc_id}/flush` - 刷新到GCS 