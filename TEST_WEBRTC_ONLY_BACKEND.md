# 测试纯 WebRTC 模式 - 后端只启动信令服务器

## 🎯 目标

启动一个**只提供 WebRTC 信令**的轻量级服务器，不启动 Yjs WebSocket 同步，测试纯 P2P 功能。

## 🚀 快速开始

### 1. 启动纯信令服务器

```bash
cd server/websocket
cargo run --bin signaling-only
```

你会看到：
```
Starting WebRTC signaling server on 0.0.0.0:8000
WebRTC Signaling endpoint available at ws://0.0.0.0:8000/signaling
Ready to accept WebRTC connections!
```

注意：**没有** "WebSocket endpoint available" 的日志！

### 2. 配置前端使用信令服务器

编辑 `ui/public/reearth_config.json`:

```json
{
  "api": "http://localhost:8080",
  "websocket": "ws://localhost:8000",
  "enableWebRTC": true,
  "devMode": true,
  "tosUrl": "https://reearth.io/docs/terms-of-service",
  "documentationUrl": "https://docs.reearth.io/"
}
```

**关键**: `websocket` 字段会被用来推导信令服务器地址：
- `ws://localhost:8000` → 信令地址 `ws://localhost:8000/signaling`

### 3. 启动前端

```bash
cd ui
yarn start
```

### 4. 测试 P2P 同步

1. 打开浏览器标签页 A
2. 打开浏览器标签页 B
3. 在浏览器控制台检查：

```javascript
// 应该看到 WebSocket 连接失败（预期的）
console.log('WebSocket provider:', yWebSocketProvider);
console.log('WebSocket connected:', yWebSocketProvider?.wsconnected);

// 但 WebRTC 应该成功
console.log('WebRTC provider:', yWebRTCProvider);
console.log('WebRTC peers:', yWebRTCProvider?.room?.webrtcConns?.size);
```

4. 在标签页 A 进行编辑
5. 观察标签页 B 是否同步

## ✅ 成功标准

如果纯 WebRTC 工作正常：

1. ✅ WebSocket 连接失败（控制台可能有错误，这是正常的）
2. ✅ WebRTC 显示有 peer 连接
3. ✅ 两个标签页能实时同步
4. ✅ 延迟很低（< 100ms）

## 🔍 调试

### 启用详细日志

```javascript
// 在浏览器控制台
localStorage.log = 'true';
// 刷新页面
```

### 检查信令连接

```javascript
// 查看信令 WebSocket 连接
console.log('Signaling connected:', 
  yWebRTCProvider?.signalingConns?.[0]?.connected
);
```

### 查看 P2P 连接状态

```javascript
// 查看所有 WebRTC 连接
console.log('WebRTC connections:', 
  Array.from(yWebRTCProvider?.room?.webrtcConns || new Map()).map(([peerId, conn]) => ({
    peerId,
    connected: conn.connected
  }))
);
```

## 🎨 架构对比

### 纯信令服务器（当前测试）
```
前端 A ←──── WebRTC P2P ────→ 前端 B
   ↓                            ↓
   └─── ws://server/signaling ──┘
        (只用于交换连接信息)
```

特点：
- ✅ 服务器负载极低（只转发信令消息）
- ✅ 数据完全 P2P，服务器不经手
- ❌ 无数据持久化
- ❌ 无跨服务器同步

### 完整服务器（生产环境）
```
前端 A ←──── WebRTC P2P ────→ 前端 B
   ↓                            ↓
   ├─ ws://server/signaling ────┤  (建立P2P)
   └─ ws://server/{doc_id} ─────┘  (备份数据)
               ↓
         Redis + GCS
```

特点：
- ✅ P2P 低延迟
- ✅ 数据持久化
- ✅ 跨服务器同步
- ⚠️ 服务器负载较高

## 📊 性能对比

| 指标 | 纯信令 | 完整服务器 |
|------|--------|-----------|
| 同步延迟 | < 100ms | < 100ms (P2P) |
| 服务器负载 | 极低 | 中等 |
| 数据持久化 | ❌ | ✅ |
| 服务器带宽 | 极低 | 中等 |

## 🛠️ 切换模式

### 启动纯信令服务器
```bash
cd server/websocket
cargo run --bin signaling-only
```

### 启动完整服务器
```bash
cd server/websocket
cargo run --bin websocket
# 或
cargo run
```

## 🎉 测试结果预期

如果一切正常，在纯信令模式下：
- WebSocket 同步会失败（这是预期的）
- WebRTC P2P 完全工作
- 证明 WebRTC 确实是独立的！

试试看！🚀


