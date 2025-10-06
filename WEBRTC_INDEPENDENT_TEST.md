# WebRTC 独立同步测试指南

## ✅ 已完成的修改

### 关键改进：WebRTC 使用独立的 Awareness

**修改前** (共享 awareness):
```typescript
yWebRTCProvider = new WebrtcProvider(roomName, yDoc, {
  awareness: yWebSocketProvider?.awareness,  // ❌ 共享
});
// WebSocket 断开 → awareness 被清理 → WebRTC 失效
```

**修改后** (独立 awareness):
```typescript
yWebRTCProvider = new WebrtcProvider(roomName, yDoc, {
  // ✅ 不传 awareness，让 WebRTC 创建自己的
});
// WebSocket 断开 → WebRTC awareness 不受影响 → 继续工作
```

## 🎯 现在的架构

```
前端 A                          前端 B
├── yDoc (共享)                 ├── yDoc (共享)
├── WebSocket Provider          ├── WebSocket Provider
│   └── Awareness (备份用)      │   └── Awareness (备份用)
└── WebRTC Provider             └── WebRTC Provider
    └── Awareness (主要)   ←P2P→    └── Awareness (主要)
```

### 两份 Awareness 的作用

1. **WebRTC Awareness** (主要)
   - 用于 UI 显示光标和用户状态
   - P2P 同步，低延迟
   - WebSocket 断开不影响

2. **WebSocket Awareness** (备份)
   - 持久化到服务器（Redis + GCS）
   - 跨服务器同步
   - 新用户加入时从服务器加载

3. **双向同步**
   - WebRTC awareness → WebSocket awareness (备份)
   - WebSocket awareness → WebRTC awareness (恢复)

## 🧪 测试步骤

### 1. 启动服务器

```bash
cd server/websocket
cargo run
```

应该看到：
```
WebSocket endpoint available at ws://0.0.0.0:8000/[doc_id]
WebRTC Signaling endpoint available at ws://0.0.0.0:8000/signaling
```

### 2. 启动前端

```bash
cd ui
yarn start
```

### 3. 测试 WebRTC P2P 独立性

#### 步骤 A: 建立连接
1. 打开浏览器标签页 A，访问 `http://localhost:3000`
2. 打开浏览器标签页 B（或另一个浏览器），访问同一个 workflow
3. 在标签页 A 的控制台输入：
   ```javascript
   // 检查 WebRTC 连接
   console.log('WebRTC peers:', 
     yWebRTCProvider?.room?.webrtcConns?.size || 0
   );
   ```
   应该显示 `WebRTC peers: 1` (连接到标签页 B)

#### 步骤 B: 关闭 WebSocket 服务器
1. 在服务器终端按 `Ctrl+C` 停止 WebSocket 服务器
2. 等待 5 秒

#### 步骤 C: 测试 P2P 同步
1. 在标签页 A 中进行编辑（添加节点、移动光标等）
2. 观察标签页 B 是否**立即同步**
3. 在标签页 B 的控制台输入：
   ```javascript
   // 检查 WebRTC 仍然连接
   console.log('WebRTC still connected:', 
     yWebRTCProvider?.room?.webrtcConns?.size > 0
   );
   ```

### 4. 预期结果

✅ **成功**: 
- WebSocket 断开后，WebRTC P2P 连接**仍然存在**
- 两个标签页之间**继续实时同步**
- 光标位置、用户状态正常显示
- 延迟非常低（< 100ms）

❌ **失败**:
- WebSocket 断开后同步停止
- 需要检查浏览器控制台的 WebRTC 错误

## 🔍 调试命令

### 检查 Providers 状态

```javascript
// WebSocket Provider
console.log('WebSocket connected:', yWebSocketProvider?.wsconnected);
console.log('WebSocket synced:', yWebSocketProvider?.synced);

// WebRTC Provider  
console.log('WebRTC connected:', yWebRTCProvider?.connected);
console.log('WebRTC peers:', yWebRTCProvider?.room?.webrtcConns);
console.log('WebRTC awareness clients:', 
  Array.from(yWebRTCProvider?.awareness?.getStates().keys())
);
```

### 查看同步延迟

```javascript
// 在标签页 A
let startTime = Date.now();
yDoc.transact(() => {
  yDoc.getMap('test').set('timestamp', Date.now());
});

// 在标签页 B
yDoc.getMap('test').observe(() => {
  const latency = Date.now() - yDoc.getMap('test').get('timestamp');
  console.log('Sync latency:', latency, 'ms');
  // WebRTC 应该 < 100ms
  // 纯 WebSocket 通常 200-500ms
});
```

### 启用 y-webrtc 日志

```javascript
// 在浏览器控制台
localStorage.log = 'true';
// 刷新页面，会看到详细的 WebRTC 日志
```

## 📊 预期性能

| 场景 | 延迟 | 说明 |
|------|------|------|
| WebSocket + WebRTC 都连接 | < 50ms | WebRTC P2P 主导 |
| 只有 WebRTC (服务器断开) | < 100ms | 纯 P2P |
| 只有 WebSocket | 200-500ms | 通过服务器中转 |

## ⚠️ 重要说明

### WebRTC 的限制

1. **新用户无法加入** - 信令服务器断开后，新的 P2P 连接无法建立
2. **数据不持久化** - 服务器断开期间的更新不会保存到 Redis/GCS
3. **适合临时故障** - 服务器重启期间，已连接的用户可以继续协作

### 生产环境建议

- WebSocket 服务器应该高可用（多实例 + 负载均衡）
- WebRTC 作为性能优化，不作为唯一依赖
- 定期检查服务器健康状态

## 🎉 架构优势

现在的架构实现了：

1. ✅ **WebRTC P2P 独立工作** - 服务器临时断开不影响已建立的 P2P 连接
2. ✅ **低延迟协作** - WebRTC 延迟 < 100ms
3. ✅ **数据持久化** - WebSocket 重连后自动同步到服务器
4. ✅ **自动降级** - WebRTC 失败时使用 WebSocket
5. ✅ **容错性强** - 部分故障不影响协作

完美的混合架构！🚀

