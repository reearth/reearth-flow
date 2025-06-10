import { Point2D, TrajectoryCompressor } from './index';

// 示例：如何使用轨迹压缩器
export function demonstrateTrajectoryCompression() {
  console.log('=== 轨迹压缩器演示 ===\n');

  // 创建压缩器实例
  const compressor = new TrajectoryCompressor(1.0); // epsilon = 1.0

  // 示例1: 压缩直线轨迹
  console.log('1. 直线轨迹压缩:');
  const linearPoints: Point2D[] = [
    { x: 0, y: 0, t: 0 },
    { x: 5, y: 5, t: 1 },
    { x: 10, y: 10, t: 2 },
    { x: 15, y: 15, t: 3 },
    { x: 20, y: 20, t: 4 },
  ];
  
  const linearResult = compressor.compress(linearPoints, 'linear-trajectory');
  console.log(`输入点数: ${linearPoints.length}`);
  console.log(`压缩后段数: ${linearResult.segments.length}`);
  console.log(`压缩率: ${(linearResult.compressionRatio * 100).toFixed(1)}%`);
  console.log(`总误差: ${linearResult.totalError.toFixed(3)}`);
  console.log(`段类型: ${linearResult.segments.map(s => s.type).join(', ')}\n`);

  // 示例2: 压缩曲线轨迹
  console.log('2. 曲线轨迹压缩:');
  const curvePoints: Point2D[] = Array.from({ length: 20 }, (_, i) => {
    const t = i / 4;
    return {
      x: t * 10,
      y: Math.sin(t) * 10 + 10,
      t: t,
    };
  });
  
  const curveResult = compressor.compress(curvePoints, 'curve-trajectory');
  console.log(`输入点数: ${curvePoints.length}`);
  console.log(`压缩后段数: ${curveResult.segments.length}`);
  console.log(`压缩率: ${(curveResult.compressionRatio * 100).toFixed(1)}%`);
  console.log(`总误差: ${curveResult.totalError.toFixed(3)}`);
  console.log(`段类型: ${curveResult.segments.map(s => s.type).join(', ')}\n`);

  // 示例3: 圆形轨迹
  console.log('3. 圆形轨迹压缩:');
  const circlePoints: Point2D[] = Array.from({ length: 24 }, (_, i) => {
    const angle = (i / 24) * 2 * Math.PI;
    return {
      x: Math.cos(angle) * 15 + 15,
      y: Math.sin(angle) * 15 + 15,
      t: i,
    };
  });
  
  const circleResult = compressor.compress(circlePoints, 'circle-trajectory');
  console.log(`输入点数: ${circlePoints.length}`);
  console.log(`压缩后段数: ${circleResult.segments.length}`);
  console.log(`压缩率: ${(circleResult.compressionRatio * 100).toFixed(1)}%`);
  console.log(`总误差: ${circleResult.totalError.toFixed(3)}`);
  console.log(`段类型: ${circleResult.segments.map(s => s.type).join(', ')}\n`);

  // 示例4: 位置评估
  console.log('4. 位置评估演示:');
  const testTimes = [0, 1, 2, 3, 4];
  console.log('直线轨迹在不同时间的位置:');
  for (const time of testTimes) {
    const pos = compressor.getPositionAtTime(linearResult, time);
    if (pos) {
      console.log(`  t=${time}: (${pos.x.toFixed(1)}, ${pos.y.toFixed(1)})`);
    } else {
      console.log(`  t=${time}: 超出范围`);
    }
  }
  console.log();

  // 示例5: 不同精度比较
  console.log('5. 不同精度比较:');
  const testPoints: Point2D[] = [
    { x: 0, y: 0, t: 0 },
    { x: 1, y: 1.2, t: 1 }, // 略微偏离直线
    { x: 2, y: 2.1, t: 2 }, // 略微偏离直线
    { x: 3, y: 2.8, t: 3 }, // 略微偏离直线
    { x: 4, y: 4, t: 4 },
  ];

  const precisions = [0.1, 0.5, 1.0, 2.0];
  for (const epsilon of precisions) {
    const preciseCompressor = new TrajectoryCompressor(epsilon);
    const result = preciseCompressor.compress(testPoints, `precision-${epsilon}`);
    console.log(`  epsilon=${epsilon}: ${result.segments.length}段, 误差=${result.totalError.toFixed(3)}`);
  }
  console.log();

  // 示例6: 性能测试
  console.log('6. 性能测试:');
  const largeDataset: Point2D[] = Array.from({ length: 1000 }, (_, i) => ({
    x: i + Math.random() * 0.5,
    y: Math.sin(i / 100) * 50 + 50 + Math.random() * 0.5,
    t: i,
  }));

  const startTime = performance.now();
  const performanceResult = compressor.compress(largeDataset, 'performance-test');
  const endTime = performance.now();

  console.log(`处理${largeDataset.length}个点:`);
  console.log(`  耗时: ${(endTime - startTime).toFixed(2)}ms`);
  console.log(`  压缩后段数: ${performanceResult.segments.length}`);
  console.log(`  压缩率: ${(performanceResult.compressionRatio * 100).toFixed(1)}%`);
  console.log(`  总误差: ${performanceResult.totalError.toFixed(3)}`);

  return {
    linearResult,
    curveResult,
    circleResult,
    performanceResult,
  };
}

// 生成测试数据的工具函数
export function generateTestTrajectory(type: 'linear' | 'curve' | 'circle' | 'zigzag', pointCount: number): Point2D[] {
  switch (type) {
    case 'linear':
      return Array.from({ length: pointCount }, (_, i) => ({
        x: i * 10,
        y: i * 10,
        t: i,
      }));

    case 'curve':
      return Array.from({ length: pointCount }, (_, i) => {
        const t = i / (pointCount - 1) * Math.PI * 2;
        return {
          x: t * 10,
          y: Math.sin(t) * 20 + 20,
          t: i,
        };
      });

    case 'circle':
      return Array.from({ length: pointCount }, (_, i) => {
        const angle = (i / pointCount) * 2 * Math.PI;
        return {
          x: Math.cos(angle) * 20 + 20,
          y: Math.sin(angle) * 20 + 20,
          t: i,
        };
      });

    case 'zigzag':
      return Array.from({ length: pointCount }, (_, i) => ({
        x: i * 5,
        y: (i % 2) * 20,
        t: i,
      }));

    default:
      throw new Error(`Unknown trajectory type: ${type}`);
  }
}

// 压缩质量分析
export function analyzeCompressionQuality(
  originalPoints: Point2D[],
  compressedTrajectory: ReturnType<TrajectoryCompressor['compress']>,
  compressor: TrajectoryCompressor
) {
  const sampleCount = 100;
  const timeRange = {
    start: Math.min(...originalPoints.map(p => p.t)),
    end: Math.max(...originalPoints.map(p => p.t)),
  };

  let totalError = 0;
  let maxError = 0;
  let validSamples = 0;

  for (let i = 0; i <= sampleCount; i++) {
    const t = timeRange.start + (i / sampleCount) * (timeRange.end - timeRange.start);
    const reconstructed = compressor.getPositionAtTime(compressedTrajectory, t);
    
    if (reconstructed) {
      // 找到最接近的原始点
      const nearestOriginal = originalPoints.reduce((nearest, point) => 
        Math.abs(point.t - t) < Math.abs(nearest.t - t) ? point : nearest
      );

      const error = Math.sqrt(
        Math.pow(reconstructed.x - nearestOriginal.x, 2) +
        Math.pow(reconstructed.y - nearestOriginal.y, 2)
      );

      totalError += error;
      maxError = Math.max(maxError, error);
      validSamples++;
    }
  }

  return {
    averageError: validSamples > 0 ? totalError / validSamples : 0,
    maxError,
    compressionRatio: compressedTrajectory.compressionRatio,
    segmentCount: compressedTrajectory.segments.length,
    originalPointCount: originalPoints.length,
  };
} 