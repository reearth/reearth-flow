export interface Point2D {
  x: number;
  y: number;
  t: number; // timestamp
}

export interface LinearSegment {
  type: 'linear';
  startPoint: Point2D;
  velocity: { x: number; y: number }; // velocity vector
  endTime: number;
}

export interface CubicBezierSegment {
  type: 'bezier';
  controlPoints: [Point2D, Point2D, Point2D, Point2D]; // [p0, p1, p2, p3]
  startTime: number;
  endTime: number;
}

export interface BSplineSegment {
  type: 'bspline';
  coefficients: Point2D[];
  knotVector: number[];
  degree: number;
  startTime: number;
  endTime: number
}

export interface DiscreteSegment {
  type: 'discrete';
  points: Point2D[];
}

export type TrajectorySegment = LinearSegment | CubicBezierSegment | BSplineSegment | DiscreteSegment;

export interface CompressedTrajectory {
  nodeId: string;
  segments: TrajectorySegment[];
  totalError: number;
  compressionRatio: number;
}

export class TrajectoryCompressor {
  constructor(private epsilon: number = 1.0) {}

  /**
   * 压缩轨迹点序列为函数片段
   */
  compress(points: Point2D[], nodeId: string): CompressedTrajectory {
    if (points.length < 2) {
      return {
        nodeId,
        segments: [{ type: 'discrete', points: [...points] }],
        totalError: 0,
        compressionRatio: 1.0,
      };
    }

    const segments: TrajectorySegment[] = [];
    let i = 0;
    let totalError = 0;

    while (i < points.length - 1) {
      const { segment, nextIndex, error } = this.fitBestSegment(points.slice(i));
      segments.push(segment);
      totalError += error;
      i += Math.max(nextIndex, 1);
    }

    const compressionRatio = segments.length / points.length;

    return {
      nodeId,
      segments,
      totalError,
      compressionRatio,
    };
  }

  /**
   * 从给定点开始拟合最佳片段
   */
  private fitBestSegment(points: Point2D[]): { segment: TrajectorySegment; nextIndex: number; error: number } {
    if (points.length < 2) {
      return {
        segment: { type: 'discrete', points: [...points] },
        nextIndex: points.length,
        error: 0,
      };
    }

    // 尝试直线拟合
    const linearResult = this.fitLinearSegment(points);
    if (linearResult.error <= this.epsilon && linearResult.nextIndex >= 2) {
      return linearResult;
    }

    // 尝试三次贝塞尔拟合
    const bezierResult = this.fitBezierSegment(points);
    if (bezierResult.error <= this.epsilon && bezierResult.nextIndex >= 4) {
      return bezierResult;
    }

    // 选择误差更小的拟合
    if (linearResult.error <= bezierResult.error) {
      return linearResult;
    } else {
      return bezierResult;
    }
  }

  /**
   * 拟合直线段：p(t) = p₀ + v·t
   */
  private fitLinearSegment(points: Point2D[]): { segment: LinearSegment; nextIndex: number; error: number } {
    if (points.length < 2) {
      const segment: LinearSegment = {
        type: 'linear',
        startPoint: points[0],
        velocity: { x: 0, y: 0 },
        endTime: points[0].t,
      };
      return { segment, nextIndex: 1, error: 0 };
    }

    let bestEnd = 2;
    let bestError = Infinity;
    let bestSegment = this.createLinearSegment(points[0], points[1]);

    for (let end = 2; end <= points.length; end++) {
      const segment = this.createLinearSegment(points[0], points[end - 1]);
      
      let maxError = 0;
      let errorExceeds = false;
      
      for (let j = 1; j < end - 1; j++) {
        const error = this.linearSegmentError(segment, points[j]);
        maxError = Math.max(maxError, error);
        
        if (error > this.epsilon) {
          errorExceeds = true;
          break;
        }
      }

      if (!errorExceeds) {
        bestEnd = end;
        bestError = maxError;
        bestSegment = segment;
      } else {
        break;
      }
    }

    return { segment: bestSegment, nextIndex: bestEnd, error: bestError };
  }

  /**
   * 拟合三次贝塞尔段
   */
  private fitBezierSegment(points: Point2D[]): { segment: CubicBezierSegment; nextIndex: number; error: number } {
    if (points.length < 4) {
      const segment = this.createBezierSegment(points);
      const error = this.bezierSegmentError(segment, points);
      return { segment, nextIndex: points.length, error };
    }

    let bestEnd = 4;
    let bestError = Infinity;
    let bestSegment = this.createBezierSegment(points.slice(0, 4));

    for (let end = 4; end <= points.length; end++) {
      const segment = this.createBezierSegment(points.slice(0, end));
      const error = this.bezierSegmentError(segment, points.slice(0, end));
      
      if (error <= this.epsilon) {
        bestEnd = end;
        bestError = error;
        bestSegment = segment;
      } else {
        break;
      }
    }

    return { segment: bestSegment, nextIndex: bestEnd, error: bestError };
  }

  /**
   * 创建直线段
   */
  private createLinearSegment(p0: Point2D, p1: Point2D): LinearSegment {
    const dt = p1.t - p0.t;
    const velocity = dt > 1e-10 
      ? { x: (p1.x - p0.x) / dt, y: (p1.y - p0.y) / dt }
      : { x: 0, y: 0 };
    
    return {
      type: 'linear',
      startPoint: p0,
      velocity,
      endTime: p1.t,
    };
  }

  /**
   * 创建贝塞尔段
   */
  private createBezierSegment(points: Point2D[]): CubicBezierSegment {
    if (points.length < 4) {
      // fallback: 使用端点创建简单贝塞尔曲线
      const p0 = points[0];
      const p3 = points[points.length - 1];
      return {
        type: 'bezier',
        controlPoints: [p0, p0, p3, p3],
        startTime: p0.t,
        endTime: p3.t,
      };
    }

    const startTime = points[0].t;
    const endTime = points[points.length - 1].t;
    
    const p0 = points[0];
    const p3 = points[points.length - 1];
    
    // 估算切线向量
    const tangentScale = (endTime - startTime) / 3.0;
    
    const p1: Point2D = {
      x: p0.x + ((points[1].x - p0.x) / (points[1].t - p0.t)) * tangentScale,
      y: p0.y + ((points[1].y - p0.y) / (points[1].t - p0.t)) * tangentScale,
      t: 0,
    };
    
    const n = points.length - 1;
    const p2: Point2D = {
      x: p3.x - ((p3.x - points[n-1].x) / (p3.t - points[n-1].t)) * tangentScale,
      y: p3.y - ((p3.y - points[n-1].y) / (p3.t - points[n-1].t)) * tangentScale,
      t: 0,
    };

    return {
      type: 'bezier',
      controlPoints: [p0, p1, p2, p3],
      startTime,
      endTime,
    };
  }

  /**
   * 计算点到直线段的误差
   */
  private linearSegmentError(segment: LinearSegment, point: Point2D): number {
    const predicted = this.evaluateLinearSegment(segment, point.t);
    return this.distance(predicted, point);
  }

  /**
   * 计算贝塞尔段的误差
   */
  private bezierSegmentError(segment: CubicBezierSegment, points: Point2D[]): number {
    let maxError = 0;
    for (const point of points) {
      const predicted = this.evaluateBezierSegment(segment, point.t);
      const error = this.distance(predicted, point);
      maxError = Math.max(maxError, error);
    }
    return maxError;
  }

  /**
   * 计算直线段在时间t的位置
   */
  private evaluateLinearSegment(segment: LinearSegment, t: number): Point2D {
    const dt = t - segment.startPoint.t;
    return {
      x: segment.startPoint.x + segment.velocity.x * dt,
      y: segment.startPoint.y + segment.velocity.y * dt,
      t,
    };
  }

  /**
   * 计算贝塞尔段在时间t的位置
   */
  private evaluateBezierSegment(segment: CubicBezierSegment, t: number): Point2D {
    const u = segment.endTime > segment.startTime 
      ? Math.max(0, Math.min(1, (t - segment.startTime) / (segment.endTime - segment.startTime)))
      : 0;
    
    const [p0, p1, p2, p3] = segment.controlPoints;
    const u_inv = 1 - u;
    
    const x = u_inv ** 3 * p0.x + 
              3 * u_inv ** 2 * u * p1.x +
              3 * u_inv * u ** 2 * p2.x +
              u ** 3 * p3.x;
              
    const y = u_inv ** 3 * p0.y + 
              3 * u_inv ** 2 * u * p1.y +
              3 * u_inv * u ** 2 * p2.y +
              u ** 3 * p3.y;

    return { x, y, t };
  }

  /**
   * 计算两点间距离
   */
  private distance(p1: Point2D, p2: Point2D): number {
    return Math.sqrt((p1.x - p2.x) ** 2 + (p1.y - p2.y) ** 2);
  }

  /**
   * 根据压缩轨迹获取指定时间的位置
   */
  getPositionAtTime(trajectory: CompressedTrajectory, t: number): Point2D | null {
    for (const segment of trajectory.segments) {
      const startTime = this.getSegmentStartTime(segment);
      const endTime = this.getSegmentEndTime(segment);
      
      if (t >= startTime && t <= endTime) {
        return this.evaluateSegment(segment, t);
      }
    }
    return null;
  }

  /**
   * 评估任意类型的段
   */
  private evaluateSegment(segment: TrajectorySegment, t: number): Point2D {
    switch (segment.type) {
      case 'linear':
        return this.evaluateLinearSegment(segment, t);
      case 'bezier':
        return this.evaluateBezierSegment(segment, t);
      case 'bspline':
        // TODO: 实现B-spline评估
        return { x: 0, y: 0, t };
      case 'discrete':
        return this.evaluateDiscreteSegment(segment, t);
    }
  }

  /**
   * 评估离散段（线性插值）
   */
  private evaluateDiscreteSegment(segment: DiscreteSegment, t: number): Point2D {
    const { points } = segment;
    if (points.length === 0) return { x: 0, y: 0, t };
    if (points.length === 1) return points[0];

    if (t <= points[0].t) return points[0];
    if (t >= points[points.length - 1].t) return points[points.length - 1];

    for (let i = 0; i < points.length - 1; i++) {
      if (t >= points[i].t && t <= points[i + 1].t) {
        const dt = points[i + 1].t - points[i].t;
        if (dt > 1e-10) {
          const alpha = (t - points[i].t) / dt;
          return {
            x: points[i].x + alpha * (points[i + 1].x - points[i].x),
            y: points[i].y + alpha * (points[i + 1].y - points[i].y),
            t,
          };
        } else {
          return points[i];
        }
      }
    }

    return points[0];
  }

  private getSegmentStartTime(segment: TrajectorySegment): number {
    switch (segment.type) {
      case 'linear':
        return segment.startPoint.t;
      case 'bezier':
        return segment.startTime;
      case 'bspline':
        return segment.startTime;
      case 'discrete':
        return segment.points[0]?.t ?? 0;
    }
  }

  private getSegmentEndTime(segment: TrajectorySegment): number {
    switch (segment.type) {
      case 'linear':
        return segment.endTime;
      case 'bezier':
        return segment.endTime;
      case 'bspline':
        return segment.endTime;
      case 'discrete':
        return segment.points[segment.points.length - 1]?.t ?? 0;
    }
  }
} 