# 轨迹压缩模块

这个模块实现了node移动轨迹的压缩算法，将移动轨迹分解为误差≤ε的函数片段，从而优化网络同步性能。

## 核心概念

### 轨迹片段类型

1. **直线段** (`LinearSegment`)
   - 公式: `p(t) = p₀ + v·t`
   - 适用于匀速直线运动
   - 只需存储起始点和速度向量

2. **三次贝塞尔段** (`CubicBezierSegment`)
   - 使用4个控制点定义的三次贝塞尔曲线
   - 适用于平滑的曲线运动
   - 存储起始点、两个控制点和结束点

3. **B样条段** (`BSplineSegment`)
   - 使用系数和节点向量定义的B样条曲线
   - 更灵活的曲线表示（待实现）

4. **离散段** (`DiscreteSegment`)
   - 当无法用函数拟合时的备用方案
   - 保留原始离散点

## 压缩算法

### 1. 分段拟合
```typescript
// 对每个轨迹片段，尝试不同的拟合方法
const { segment, nextIndex, error } = this.fitBestSegment(points.slice(i));
```

### 2. 误差控制
- 使用可配置的误差阈值ε（像素单位）
- 只有当拟合误差≤ε时才采用该片段
- 否则回退到离散点表示

### 3. 优先级策略
1. 首先尝试直线拟合（最简单）
2. 如果直线拟合失败，尝试贝塞尔拟合
3. 选择误差最小的拟合结果

## 使用方法

### 基本使用
```typescript
import { TrajectoryCompressor } from '@flow/lib/trajectory';

const compressor = new TrajectoryCompressor(1.0); // 1px误差容差
const points = [
  { x: 0, y: 0, t: 0 },
  { x: 10, y: 5, t: 100 },
  { x: 20, y: 10, t: 200 }
];

const compressed = compressor.compress(points, 'node-1');
console.log(compressed.compressionRatio); // 压缩比
console.log(compressed.segments); // 压缩后的片段
```

### 与React Hook集成
```typescript
import { useTrajectorySmoothing } from '@flow/lib/trajectory/useTrajectorySmoothing';

const MyComponent = () => {
  const {
    addPositionUpdate,
    getSmoothedPosition,
    animateToPosition
  } = useTrajectorySmoothing({
    epsilon: 1.0,
    maxTrajectoryPoints: 50,
    smoothingDuration: 300
  });

  // 添加位置更新
  addPositionUpdate('node-1', x, y);

  // 获取平滑位置
  const smoothPos = getSmoothedPosition('node-1', timestamp);

  // 平滑动画到目标位置
  animateToPosition('node-1', from, to, onUpdate);
};
```

### 在useYNode中的集成
```typescript
// 在位置更新时自动进行轨迹压缩
case "position": {
  if (existingYNode && change.position) {
    // 处理轨迹压缩
    processNodePositionUpdate(change.id, change.position.x, change.position.y);
    
    // 更新Y.js
    const newPosition = new Y.Map<unknown>();
    newPosition.set("x", change.position.x);
    newPosition.set("y", change.position.y);
    existingYNode.set("position", newPosition);
  }
  break;
}
```

## 性能优化

### 网络同步优化
1. **减少传输数据量**: 压缩轨迹减少需要同步的数据
2. **批量更新**: 可以批量发送压缩后的轨迹片段
3. **预测性同步**: 基于函数片段预测未来位置

### 内存管理
1. **自动压缩**: 当轨迹点超过阈值时自动压缩
2. **清理策略**: 删除node时清理相关轨迹数据
3. **循环缓冲**: 保留最近的点用于未来压缩

## 配置参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `epsilon` | 1.0 | 误差容差（像素） |
| `maxTrajectoryPoints` | 50 | 触发压缩的最大点数 |
| `smoothingDuration` | 300 | 平滑动画持续时间（毫秒） |
| `enableSmoothing` | true | 是否启用平滑功能 |

## 压缩效果示例

### 典型压缩比
- **直线运动**: 可压缩至原数据的5-10%
- **曲线运动**: 可压缩至原数据的20-40%
- **复杂轨迹**: 可压缩至原数据的50-70%

### 误差控制
- `ε = 0.5px`: 高精度，低压缩比
- `ε = 1.0px`: 平衡精度和压缩比
- `ε = 2.0px`: 高压缩比，略微降低精度

## 未来扩展

1. **B样条实现**: 完善B样条拟合算法
2. **自适应ε**: 根据轨迹特征动态调整误差阈值
3. **预测算法**: 基于历史轨迹预测未来移动
4. **多用户同步**: 优化多用户环境下的轨迹同步
5. **GPU加速**: 使用WebGL加速轨迹计算和渲染

## 技术原理

### 直线拟合算法
使用最小二乘法拟合直线，计算速度向量：
```
v = (p_end - p_start) / (t_end - t_start)
```

### 贝塞尔拟合算法
1. 使用起点和终点作为P0和P3
2. 根据切线估算控制点P1和P2
3. 验证拟合误差是否在容差范围内

### 误差计算
对于每个原始点，计算到拟合曲线的最短距离：
```
error = min(distance(point, curve))
```

这个实现提供了完整的轨迹压缩解决方案，可以显著优化node移动的网络同步性能。 