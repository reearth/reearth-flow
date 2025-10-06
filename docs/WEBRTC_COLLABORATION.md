# WebRTC 协作功能

## 概述

Re:Earth Flow 现在支持 **WebRTC P2P + WebSocket 备份** 的混合协作架构。这种架构结合了 WebRTC 的低延迟优势和 WebSocket 的可靠性，为实时协作提供了最佳体验。

## 架构说明

```
┌─────────┐     WebRTC P2P    ┌─────────┐
│ Client1 │◄─────────────────►│ Client2 │
└────┬────┘                   └────┬────┘
     │                             │
     │    WebSocket (backup)       │
     └──────────┬──────────┬───────┘
                │          │
         ┌──────▼──────────▼──────┐
         │  WebSocket Server      │
         │  (Redis + GCS backup)  │
         │  + Signaling Server    │
         └────────────────────────┘
```

### 工作原理

1. **WebRTC P2P 连接**: 客户端之间直接建立点对点连接，实现低延迟的实时同步
2. **WebSocket 备份**: 同时维护到服务器的 WebSocket 连接，确保数据持久化到 Redis 和 GCS
3. **信令服务器**: WebSocket 服务器同时提供 WebRTC 信令功能，帮助客户端建立 P2P 连接

## 优势

- ⚡ **更快的实时同步**: WebRTC 提供 P2P 连接，延迟更低（通常 < 100ms）
- 🚀 **减轻服务器负担**: 大部分同步流量走 P2P，服务器只处理备份和信令
- 📈 **更好的扩展性**: 客户端之间直接连接，服务器压力随用户增长线性增长而非指数增长
- 🔒 **数据安全**: WebSocket 仍然连接到后端，所有数据都备份到 Redis 和 GCS
- 🛡️ **渐进式增强**: 如果 WebRTC 连接失败（防火墙/NAT），仍然可以通过 WebSocket 工作

## 配置

### 前端配置

在 `reearth_config.json` 中添加：

```json
{
  "websocket": "ws://localhost:8000",
  "enableWebRTC": true
}
```

### 环境变量

生产环境中，可以通过环境变量配置：

```bash
FLOW_WEBSOCKET=wss://your-server.com
FLOW_ENABLE_WEBRTC=true
```

### 后端配置

WebSocket 服务器会自动在以下端点提供服务：

- WebSocket 同步: `ws://localhost:8000/{doc_id}`
- WebRTC 信令: `ws://localhost:8000/signaling`

## 使用方法

### 基本使用

不需要修改任何代码，现有的 `useYjsSetup` hook 会自动使用 WebRTC：

```typescript
const { yWorkflows, isSynced, yAwareness } = useYjsSetup({
  workflowId: "workflow-123",
  projectId: "project-456",
  isProtected: true,
});
```

### 禁用 WebRTC

如果需要临时禁用 WebRTC（例如测试），可以：

1. **配置文件禁用**:
```json
{
  "enableWebRTC": false
}
```

2. **代码中禁用**:
```typescript
const { yWorkflows } = useYjsSetup({
  workflowId: "workflow-123",
  projectId: "project-456",
  enableWebRTC: false, // 覆盖配置文件
});
```

## 网络要求

### STUN 服务器

默认使用 Google 的公共 STUN 服务器：
- `stun:stun.l.google.com:19302`
- `stun:stun1.l.google.com:19302`

### TURN 服务器（可选）

对于严格的企业防火墙环境，可能需要配置 TURN 服务器。在 `useYjsSetup.ts` 中添加：

```typescript
peerOpts: {
  config: {
    iceServers: [
      { urls: "stun:stun.l.google.com:19302" },
      {
        urls: "turn:your-turn-server.com",
        username: "username",
        credential: "password"
      }
    ]
  }
}
```

## 端口要求

- WebSocket 服务器: 默认 `8000`
- WebRTC 使用随机 UDP 端口（由浏览器自动分配）

## 故障排除

### WebRTC 连接失败

如果 WebRTC 连接失败，系统会自动降级到纯 WebSocket 模式。检查：

1. 浏览器控制台是否有 ICE 连接错误
2. 防火墙是否阻止 UDP 流量
3. 是否需要配置 TURN 服务器

### 查看连接状态

在浏览器控制台中：

```javascript
// 查看 awareness 状态
console.log(yAwareness?.getStates());

// 查看当前连接的客户端数量
console.log(yAwareness?.getStates().size);
```

## 性能监控

WebRTC 连接会显著降低延迟：

- **纯 WebSocket**: 200-500ms（取决于服务器位置）
- **WebRTC P2P**: 50-150ms（取决于客户端之间的网络）

## 安全性

- WebRTC 连接使用 DTLS-SRTP 加密
- WebSocket 连接支持 TLS (wss://)
- 认证通过 WebSocket 连接的 token 参数进行

## 开发说明

### 依赖包

前端新增依赖：
- `y-webrtc`: 10.3.0

后端无需新增依赖（使用现有的 Axum WebSocket）

### 测试

测试 WebRTC 功能：

```bash
# 启动后端
cd server/websocket
cargo run

# 启动前端
cd ui
yarn start

# 在两个不同的浏览器标签页中打开同一个 workflow
# 应该能看到实时同步
```

## 参考资料

- [Yjs 官方文档](https://docs.yjs.dev/)
- [y-webrtc GitHub](https://github.com/yjs/y-webrtc)
- [WebRTC API](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API)

