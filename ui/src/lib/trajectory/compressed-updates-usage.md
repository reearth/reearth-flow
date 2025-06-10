# 轨迹压缩更新机制

## 概述

新的轨迹压缩更新机制通过以下方式优化了节点位置更新的性能：

1. **节流更新**：避免过于频繁的awareness同步
2. **轨迹压缩**：在达到阈值时使用压缩算法优化位置数据
3. **智能调度**：根据情况选择最适合的更新策略

## 工作原理

### 1. 节流更新 (Throttled Updates)

```typescript
// 位置更新不再立即发送到awareness
// 而是在100ms内节流合并多次更新
const UPDATE_THROTTLE_MS = 100;

// 每次位置变化时，会：
// 1. 记录到pending updates
// 2. 设置/重置100ms定时器
// 3. 定时器到期时才真正更新awareness
```

### 2. 轨迹压缩更新

```typescript
// 当轨迹点达到50个时
const MAX_TRAJECTORY_POINTS = 50;

// 系统会：
// 1. 压缩轨迹为函数段
// 2. 从压缩轨迹计算最新位置
// 3. 立即更新awareness（绕过节流）
```

### 3. 拖拽结束处理

```typescript
// 可以手动flush所有pending updates
const { flushPendingUpdates } = useYNode(/* ... */);

// 在拖拽结束时调用
onDragEnd(() => {
  flushPendingUpdates();
});
```

## 性能优势

### 网络传输优化
- **原来**：每次鼠标移动都发送一次更新
- **现在**：100ms内的多次移动合并为一次更新

### 数据压缩
- **原来**：存储所有原始轨迹点
- **现在**：将50个点压缩为几个函数段

### 实时性保持
- 重要的轨迹变化（压缩阈值）立即同步
- 拖拽结束时强制同步确保最终一致性

## 配置参数

```typescript
// 压缩误差容忍度（像素）
const trajectoryCompressor = new TrajectoryCompressor(1.0);

// 节流时间间隔
const UPDATE_THROTTLE_MS = 100;

// 压缩触发阈值
const MAX_TRAJECTORY_POINTS = 50;

// 压缩后保留的点数
const keepRecentPoints = 10;
```

## 使用示例

### 基本用法

```typescript
// 在useYNode中自动处理，无需手动调用
const {
  handleYNodesChange,
  flushPendingUpdates,
  getInterpolatedPosition,
} = useYNode({
  currentYWorkflow,
  // ... 其他参数
});

// React Flow会自动调用handleYNodesChange
<ReactFlow onNodesChange={handleYNodesChange} />
```

### 手动刷新更新

```typescript
// 在特定时机强制刷新所有pending updates
const handleDragEnd = () => {
  // 确保拖拽结束时位置同步
  flushPendingUpdates();
};
```

### 获取插值位置

```typescript
// 基于压缩轨迹获取任意时间的位置
const position = getInterpolatedPosition(nodeId, timestamp);
if (position) {
  console.log(`Node at (${position.x}, ${position.y}) at time ${position.t}`);
}
```

## 技术细节

### 更新策略选择

```typescript
const shouldUseCompressed = processNodePositionUpdate(nodeId, x, y);

if (shouldUseCompressed) {
  // 轨迹已压缩，使用压缩数据更新
  applyCompressedUpdate(nodeId, yNodes);
} else {
  // 正常情况，使用节流更新
  scheduleThrottledUpdate(nodeId, yNodes);
}
```

### 内存管理

```typescript
// 删除节点时自动清理
clearTrajectoryData(nodeId); // 清理轨迹数据
// 同时清理：
// - 原始轨迹点
// - 压缩轨迹
// - pending updates
// - 定时器
```

## 监控和调试

### 压缩日志

```typescript
// 轨迹压缩时会输出日志
console.log(`Compressed trajectory for node ${nodeId}, compression ratio: ${ratio}`);
```

### 性能指标

- 压缩率：压缩后段数 / 原始点数
- 更新频率：实际awareness更新次数 vs 位置变化次数
- 内存使用：轨迹点数量和压缩数据大小

## 最佳实践

1. **拖拽结束时刷新**：确保最终位置一致性
2. **监控压缩率**：调整epsilon参数优化压缩效果
3. **适当的节流时间**：平衡实时性和性能
4. **及时清理**：避免内存泄漏

## 兼容性

该机制与现有的Yjs协作系统完全兼容：
- 不影响多用户实时协作
- 保持轨迹数据的时间一致性
- 向后兼容原有的位置更新API 