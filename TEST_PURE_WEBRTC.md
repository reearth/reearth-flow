# 测试纯 WebRTC P2P 同步

## 🎯 目标

测试 WebRTC 是否真正独立工作，完全不依赖你的 WebSocket 服务器。

## 📋 测试步骤

### 方式 1: 使用公共信令服务器（最简单）

#### 1. 修改配置

编辑 `ui/public/reearth_config.json`，**注释掉或删除 `websocket` 字段**：

```json
{
  "api": "http://localhost:8080",
  "enableWebRTC": true,
  "devMode": true,
  "tosUrl": "https://reearth.io/docs/terms-of-service",
  "documentationUrl": "https://docs.reearth.io/"
}
```

注意：**没有 `"websocket"` 字段**！

#### 2. 启动前端（不需要启动 WebSocket 服务器）

```bash
cd ui
yarn start
```

#### 3. 测试

1. 打开浏览器标签页 A: `http://localhost:3000`
2. 打开浏览器标签页 B: `http://localhost:3000`（或另一个浏览器）
3. 打开同一个 workflow
4. 在浏览器控制台查看：

```javascript
// 检查 WebRTC 连接
console.log('WebRTC provider:', yWebRTCProvider);
console.log('WebRTC connected:', yWebRTCProvider?.connected);
console.log('WebRTC peers:', yWebRTCProvider?.room?.webrtcConns);
console.log('WebSocket provider:', yWebSocketProvider);  // 应该是 null
```

5. 在标签页 A 进行编辑
6. 观察标签页 B 是否实时同步

### 方式 2: 只启动信令服务器（测试你的后端）

#### 1. 只启动信令服务器

```bash
cd server/websocket
cargo run
```

#### 2. 修改配置指向本地信令服务器

`ui/public/reearth_config.json`:
```json
{
  "api": "http://localhost:8080",
  "websocket": "ws://localhost:8000",
  "enableWebRTC": true,
  "devMode": true
}
```

#### 3. 在浏览器控制台模拟 WebSocket 断开

```javascript
// 强制断开 WebSocket（只保留 WebRTC）
if (yWebSocketProvider) {
  yWebSocketProvider.disconnect();
  yWebSocketProvider.destroy();
}

// 检查 WebRTC 是否仍然工作
console.log('WebRTC still connected:', yWebRTCProvider?.connected);
console.log('WebRTC peers:', yWebRTCProvider?.room?.webrtcConns?.size);
```

#### 4. 测试同步

在标签页 A 编辑，标签页 B 应该仍能接收更新。

## ✅ 成功标准

如果 WebRTC 正常工作，你应该看到：

1. ✅ 浏览器控制台显示 `WebRTC peers: 1`（或更多）
2. ✅ 标签页之间实时同步（< 100ms 延迟）
3. ✅ **不需要 WebSocket 服务器**或**WebSocket 断开后仍能同步**
4. ✅ 光标位置实时显示

## ❌ 失败排查

### 问题 1: WebRTC peers 为 0

**原因**: P2P 连接未建立

**检查**:
```javascript
console.log('Signaling URLs:', yWebRTCProvider?.signalingUrls);
console.log('Room:', yWebRTCProvider?.room);
```

**可能的解决方案**:
- 检查防火墙是否阻止 WebRTC
- 检查浏览器控制台的 ICE 连接错误
- 尝试使用不同的浏览器

### 问题 2: 无法建立 P2P 连接

**原因**: 网络环境限制（NAT/防火墙）

**解决方案**: 使用 TURN 服务器（需要额外配置）

### 问题 3: 仍然需要 WebSocket

**原因**: 代码可能还在等待 WebSocket sync

**检查**: 查看 `isSynced` 状态
```javascript
console.log('Is synced:', isSynced);
```

## 🎨 架构对比

### 纯 WebRTC 模式
```
前端 A ←─ WebRTC P2P ─→ 前端 B
         ↑
         └─ 公共信令服务器（只用于建立连接）
```

### 混合模式（推荐）
```
前端 A ←─ WebRTC P2P ─→ 前端 B
   ↓                      ↓
   └─ WebSocket 备份 ─────┘
            ↓
      你的服务器（Redis + GCS）
```

## 📊 性能对比

| 模式 | 延迟 | 持久化 | 服务器依赖 |
|------|------|--------|-----------|
| 纯 WebRTC | < 100ms | ❌ | 仅信令 |
| 纯 WebSocket | 200-500ms | ✅ | 完全依赖 |
| **混合模式** | **< 100ms** | **✅** | **部分** |

## 🚀 推荐测试流程

1. **先测试纯 WebRTC** (方式 1) 
   - 验证 P2P 功能正常
   - 确认延迟低

2. **再测试混合模式** (方式 2)
   - 验证数据持久化
   - 测试服务器断开后 P2P 继续工作

3. **最后测试容错** 
   - 模拟各种故障场景
   - 验证降级策略

好运！🎉


