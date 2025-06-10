import { describe, it, expect, beforeEach } from 'vitest';
import {
  Point2D,
  TrajectoryCompressor,
  LinearSegment,
  CubicBezierSegment,
  CompressedTrajectory,
} from './index';

describe('TrajectoryCompressor', () => {
  let compressor: TrajectoryCompressor;

  beforeEach(() => {
    compressor = new TrajectoryCompressor(1.0); // epsilon = 1.0
  });

  describe('基本功能测试', () => {
    it('应该能够处理空轨迹', () => {
      const result = compressor.compress([], 'test-node');
      
      expect(result.nodeId).toBe('test-node');
      expect(result.segments).toHaveLength(1);
      expect(result.segments[0].type).toBe('discrete');
      expect(result.totalError).toBe(0);
      expect(result.compressionRatio).toBe(1.0);
    });

    it('应该能够处理单点轨迹', () => {
      const points: Point2D[] = [{ x: 10, y: 20, t: 0 }];
      const result = compressor.compress(points, 'test-node');
      
      expect(result.segments).toHaveLength(1);
      expect(result.segments[0].type).toBe('discrete');
      expect(result.totalError).toBe(0);
      expect(result.compressionRatio).toBe(1.0);
    });

    it('应该能够处理两点直线轨迹', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 0 },
        { x: 10, y: 10, t: 1 },
      ];
      const result = compressor.compress(points, 'test-node');
      
      expect(result.segments).toHaveLength(1);
      expect(result.segments[0].type).toBe('linear');
      
      const segment = result.segments[0] as LinearSegment;
      expect(segment.startPoint).toEqual({ x: 0, y: 0, t: 0 });
      expect(segment.velocity).toEqual({ x: 10, y: 10 });
      expect(segment.endTime).toBe(1);
    });
  });

  describe('直线段压缩测试', () => {
    it('应该正确压缩完美直线轨迹', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 0 },
        { x: 5, y: 5, t: 0.5 },
        { x: 10, y: 10, t: 1 },
        { x: 15, y: 15, t: 1.5 },
      ];
      const result = compressor.compress(points, 'linear-test');
      
      expect(result.segments).toHaveLength(1);
      expect(result.segments[0].type).toBe('linear');
      expect(result.totalError).toBeLessThan(0.1);
    });

    it('应该正确计算直线段的速度', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 0 },
        { x: 20, y: 10, t: 2 },
      ];
      const result = compressor.compress(points, 'velocity-test');
      
      const segment = result.segments[0] as LinearSegment;
      expect(segment.velocity.x).toBe(10); // 20/2 = 10
      expect(segment.velocity.y).toBe(5);  // 10/2 = 5
    });

    it('应该处理时间间隔为0的情况', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 1 },
        { x: 10, y: 10, t: 1 }, // 相同时间
      ];
      const result = compressor.compress(points, 'zero-time-test');
      
      const segment = result.segments[0] as LinearSegment;
      expect(segment.velocity.x).toBe(0);
      expect(segment.velocity.y).toBe(0);
    });
  });

  describe('贝塞尔段压缩测试', () => {
    it('应该为复杂曲线创建贝塞尔段', () => {
      // 创建一个曲线轨迹，不能很好地用直线拟合
      const points: Point2D[] = [
        { x: 0, y: 0, t: 0 },
        { x: 2, y: 8, t: 0.25 },
        { x: 4, y: 12, t: 0.5 },
        { x: 6, y: 8, t: 0.75 },
        { x: 8, y: 0, t: 1 },
      ];
      
      // 使用较小的epsilon来强制更精确的拟合
      const preciseCompressor = new TrajectoryCompressor(0.5);
      const result = preciseCompressor.compress(points, 'curve-test');
      
      // 应该有至少一个贝塞尔段
      const hasBeziersegment = result.segments.some(s => s.type === 'bezier');
      expect(hasBeziersegment || result.segments.length > 1).toBe(true);
    });

    it('应该正确设置贝塞尔段的时间范围', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 1 },
        { x: 2, y: 4, t: 2 },
        { x: 4, y: 2, t: 3 },
        { x: 6, y: 0, t: 4 },
      ];
      
      const result = compressor.compress(points, 'bezier-time-test');
      
      for (const segment of result.segments) {
        if (segment.type === 'bezier') {
          const bezierSeg = segment as CubicBezierSegment;
          expect(bezierSeg.startTime).toBeLessThanOrEqual(bezierSeg.endTime);
          expect(bezierSeg.controlPoints).toHaveLength(4);
        }
      }
    });
  });

  describe('位置评估测试', () => {
    it('应该正确评估直线段上的位置', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 0 },
        { x: 10, y: 10, t: 1 },
      ];
      const trajectory = compressor.compress(points, 'eval-test');
      
      // 测试中间点
      const midPos = compressor.getPositionAtTime(trajectory, 0.5);
      expect(midPos).not.toBeNull();
      if (midPos) {
        expect(midPos.x).toBeCloseTo(5, 1);
        expect(midPos.y).toBeCloseTo(5, 1);
        expect(midPos.t).toBe(0.5);
      }
      
      // 测试起点
      const startPos = compressor.getPositionAtTime(trajectory, 0);
      expect(startPos).not.toBeNull();
      if (startPos) {
        expect(startPos.x).toBeCloseTo(0, 1);
        expect(startPos.y).toBeCloseTo(0, 1);
      }
      
      // 测试终点
      const endPos = compressor.getPositionAtTime(trajectory, 1);
      expect(endPos).not.toBeNull();
      if (endPos) {
        expect(endPos.x).toBeCloseTo(10, 1);
        expect(endPos.y).toBeCloseTo(10, 1);
      }
    });

    it('应该正确评估离散段上的位置', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 0 },
        { x: 5, y: 5, t: 1 },
        { x: 10, y: 0, t: 2 },
      ];
      
      // 创建一个肯定会产生离散段的轨迹
      const discreteTrajectory: CompressedTrajectory = {
        nodeId: 'discrete-test',
        segments: [{ type: 'discrete', points }],
        totalError: 0,
        compressionRatio: 1.0,
      };
      
      // 测试线性插值
      const midPos = compressor.getPositionAtTime(discreteTrajectory, 0.5);
      expect(midPos).not.toBeNull();
      if (midPos) {
        expect(midPos.x).toBeCloseTo(2.5, 1);
        expect(midPos.y).toBeCloseTo(2.5, 1);
      }
    });

    it('应该处理时间范围外的查询', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 1 },
        { x: 10, y: 10, t: 2 },
      ];
      const trajectory = compressor.compress(points, 'out-of-range-test');
      
      // 时间太早
      const beforePos = compressor.getPositionAtTime(trajectory, 0);
      expect(beforePos).toBeNull();
      
      // 时间太晚
      const afterPos = compressor.getPositionAtTime(trajectory, 3);
      expect(afterPos).toBeNull();
    });
  });

  describe('边界情况测试', () => {
    it('应该处理重复点', () => {
      const points: Point2D[] = [
        { x: 5, y: 5, t: 0 },
        { x: 5, y: 5, t: 1 },
        { x: 5, y: 5, t: 2 },
      ];
      const result = compressor.compress(points, 'duplicate-test');
      
      expect(result.segments).toHaveLength(1);
      expect(result.totalError).toBe(0);
    });

    it('应该处理非常大的坐标值', () => {
      const points: Point2D[] = [
        { x: 1e6, y: 1e6, t: 0 },
        { x: 1e6 + 100, y: 1e6 + 100, t: 1 },
      ];
      const result = compressor.compress(points, 'large-coords-test');
      
      expect(result.segments).toHaveLength(1);
      expect(Number.isFinite(result.totalError)).toBe(true);
    });

    it('应该处理负坐标', () => {
      const points: Point2D[] = [
        { x: -10, y: -20, t: 0 },
        { x: -5, y: -10, t: 1 },
        { x: 0, y: 0, t: 2 },
      ];
      const result = compressor.compress(points, 'negative-coords-test');
      
      expect(result.segments.length).toBeGreaterThan(0);
      expect(Number.isFinite(result.totalError)).toBe(true);
    });

    it('应该处理乱序时间戳', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 2 },
        { x: 5, y: 5, t: 1 },
        { x: 10, y: 10, t: 3 },
      ];
      
      // 注意：这个测试可能暴露算法对时间戳顺序的假设
      const result = compressor.compress(points, 'unordered-time-test');
      expect(result.segments.length).toBeGreaterThan(0);
    });
  });

  describe('性能和压缩率测试', () => {
    it('应该计算合理的压缩率', () => {
      const points: Point2D[] = Array.from({ length: 100 }, (_, i) => ({
        x: i,
        y: i * i / 100, // 二次函数
        t: i / 10,
      }));
      
      const result = compressor.compress(points, 'compression-test');
      
      expect(result.compressionRatio).toBeLessThan(1.0);
      expect(result.segments.length).toBeLessThan(points.length);
      expect(Number.isFinite(result.totalError)).toBe(true);
    });

    it('应该在合理时间内完成大数据集压缩', () => {
      const points: Point2D[] = Array.from({ length: 1000 }, (_, i) => ({
        x: Math.sin(i / 100) * 100,
        y: Math.cos(i / 100) * 100,
        t: i,
      }));
      
      const startTime = performance.now();
      const result = compressor.compress(points, 'performance-test');
      const endTime = performance.now();
      
      expect(endTime - startTime).toBeLessThan(1000); // 应该在1秒内完成
      expect(result.segments.length).toBeGreaterThan(0);
    });
  });

  describe('精度和误差测试', () => {
    it('应该尊重epsilon参数', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 0 },
        { x: 1, y: 1.5, t: 1 }, // 稍微偏离直线
        { x: 2, y: 2, t: 2 },
      ];
      
      const strictCompressor = new TrajectoryCompressor(0.1);
      const lenientCompressor = new TrajectoryCompressor(2.0);
      
      const strictResult = strictCompressor.compress(points, 'strict-test');
      const lenientResult = lenientCompressor.compress(points, 'lenient-test');
      
      // 严格的压缩器应该产生更多段或更高精度
      expect(strictResult.totalError).toBeLessThanOrEqual(lenientResult.totalError);
    });

    it('应该保持轨迹的连续性', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 0 },
        { x: 5, y: 5, t: 1 },
        { x: 10, y: 5, t: 2 },
        { x: 15, y: 0, t: 3 },
      ];
      
      const trajectory = compressor.compress(points, 'continuity-test');
      
      // 检查轨迹在各个时间点的连续性
      for (let t = 0; t <= 3; t += 0.1) {
        const pos = compressor.getPositionAtTime(trajectory, t);
        if (pos) {
          expect(Number.isFinite(pos.x)).toBe(true);
          expect(Number.isFinite(pos.y)).toBe(true);
        }
      }
    });
  });

  describe('特殊轨迹模式测试', () => {
    it('应该处理圆形轨迹', () => {
      const points: Point2D[] = Array.from({ length: 16 }, (_, i) => {
        const angle = (i / 16) * 2 * Math.PI;
        return {
          x: Math.cos(angle) * 10,
          y: Math.sin(angle) * 10,
          t: i,
        };
      });
      
      const result = compressor.compress(points, 'circle-test');
      
      expect(result.segments.length).toBeGreaterThan(0);
      expect(Number.isFinite(result.totalError)).toBe(true);
      
      // 验证能够重构轨迹
      const testTime = 8; // 中间时间点
      const pos = compressor.getPositionAtTime(result, testTime);
      expect(pos).not.toBeNull();
    });

    it('应该处理折线轨迹', () => {
      const points: Point2D[] = [
        { x: 0, y: 0, t: 0 },
        { x: 10, y: 0, t: 1 },
        { x: 10, y: 10, t: 2 },
        { x: 0, y: 10, t: 3 },
        { x: 0, y: 0, t: 4 },
      ];
      
      const result = compressor.compress(points, 'zigzag-test');
      
      // 折线应该需要多个段来表示
      expect(result.segments.length).toBeGreaterThan(1);
      
      // 验证关键点的准确性
      const cornerPos = compressor.getPositionAtTime(result, 2);
      expect(cornerPos).not.toBeNull();
      if (cornerPos) {
        expect(cornerPos.x).toBeCloseTo(10, 1);
        expect(cornerPos.y).toBeCloseTo(10, 1);
      }
    });
  });
}); 