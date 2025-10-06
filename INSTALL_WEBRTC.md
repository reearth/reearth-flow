# WebRTC 功能安装指南

## 快速开始

### 1. 安装前端依赖

```bash
cd ui
yarn install
```

这会自动安装新添加的 `y-webrtc@10.3.0` 包。

### 2. 启动后端服务

```bash
cd server/websocket
cargo run
```

你应该看到以下输出：
```
Starting server on 0.0.0.0:8000
WebSocket endpoint available at ws://0.0.0.0:8000/[doc_id]
WebRTC Signaling endpoint available at ws://0.0.0.0:8000/signaling
HTTP API endpoints available at http://0.0.0.0:8000/api/document/...
```

### 3. 启动前端开发服务器

```bash
cd ui
yarn start
```

### 4. 测试 WebRTC 功能

1. 在浏览器中打开 `http://localhost:3000`
2. 创建或打开一个 workflow
3. 在另一个浏览器标签页（或不同的浏览器）中打开同一个 workflow
4. 在任一标签页中进行编辑，你应该能看到实时同步

### 5. 验证 WebRTC 连接

打开浏览器开发者工具（F12），在控制台中输入：

```javascript
// 查看当前连接的客户端数量
console.log('Connected peers:', yAwareness?.getStates().size - 1);

// 查看所有连接的客户端
console.log('All peers:', Array.from(yAwareness?.getStates().values()));
```

如果 WebRTC 工作正常，你应该能看到其他连接的客户端。

## 配置选项

### 开发环境配置

编辑 `ui/public/reearth_config.json`:

```json
{
  "websocket": "ws://localhost:8000",
  "enableWebRTC": true
}
```

### 生产环境配置

使用环境变量：

```bash
export FLOW_WEBSOCKET=wss://your-server.com
export FLOW_ENABLE_WEBRTC=true
```

## 故障排除

### 问题 1: 找不到 y-webrtc 模块

**错误**: `Cannot find module 'y-webrtc'`

**解决方案**: 
```bash
cd ui
rm -rf node_modules
yarn install
```

### 问题 2: WebRTC 连接失败

**现象**: 浏览器控制台显示 ICE connection failed

**可能原因**:
- 防火墙阻止 UDP 流量
- 严格的企业网络环境

**解决方案**: 
暂时禁用 WebRTC，使用纯 WebSocket 模式：
```json
{
  "enableWebRTC": false
}
```

### 问题 3: 后端信令服务器未启动

**错误**: WebSocket 连接到 `/signaling` 失败

**解决方案**:
确保使用最新的后端代码并重新编译：
```bash
cd server/websocket
cargo clean
cargo build --release
cargo run --release
```

### 问题 4: 编译错误

**错误**: Rust 编译错误

**解决方案**:
```bash
cd server/websocket
cargo update
cargo build
```

## 验证安装

### 检查前端依赖

```bash
cd ui
yarn list --pattern y-webrtc
```

应该显示：
```
└─ y-webrtc@10.3.0
```

### 检查后端路由

访问 `http://localhost:8000/api/document` 应该返回 404（正常，因为需要文档 ID）

尝试连接 WebSocket:
```bash
# 测试 Yjs 同步端点
wscat -c ws://localhost:8000/test-doc

# 测试信令端点
wscat -c ws://localhost:8000/signaling
```

## 性能测试

### 测量延迟

在浏览器控制台中：

```javascript
let startTime = Date.now();
yDoc.transact(() => {
  yDoc.getMap('test').set('timestamp', Date.now());
});

// 在另一个客户端监听变化
yDoc.getMap('test').observe(() => {
  console.log('Latency:', Date.now() - yDoc.getMap('test').get('timestamp'), 'ms');
});
```

### 预期结果

- **WebRTC P2P**: 50-150ms
- **纯 WebSocket**: 200-500ms

## 下一步

- 阅读 [WEBRTC_IMPLEMENTATION.md](./WEBRTC_IMPLEMENTATION.md) 了解实现细节
- 阅读 [docs/WEBRTC_COLLABORATION.md](./docs/WEBRTC_COLLABORATION.md) 了解使用方法

## 需要帮助？

如果遇到问题：
1. 检查浏览器控制台的错误信息
2. 检查后端日志输出
3. 确保防火墙允许 WebSocket 连接
4. 尝试在 Chrome/Firefox 最新版本中测试

