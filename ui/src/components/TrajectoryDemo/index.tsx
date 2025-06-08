import React, { useState, useEffect, useRef } from 'react';
import { TrajectoryCompressor, Point2D, CompressedTrajectory } from '@flow/lib/trajectory';

interface TrajectoryDemoProps {
  className?: string;
}

export const TrajectoryDemo: React.FC<TrajectoryDemoProps> = ({ className }) => {
  const [trajectory, setTrajectory] = useState<Point2D[]>([]);
  const [compressedTrajectory, setCompressedTrajectory] = useState<CompressedTrajectory | null>(null);
  const [isRecording, setIsRecording] = useState(false);
  const [epsilon, setEpsilon] = useState(2.0);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const compressorRef = useRef(new TrajectoryCompressor(2.0));

  useEffect(() => {
    compressorRef.current = new TrajectoryCompressor(epsilon);
  }, [epsilon]);

  useEffect(() => {
    drawTrajectory();
  }, [trajectory, compressedTrajectory]);

  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!canvasRef.current) return;
    
    const rect = canvasRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const t = Date.now();
    
    setTrajectory([{ x, y, t }]);
    setCompressedTrajectory(null);
    setIsRecording(true);
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!isRecording || !canvasRef.current) return;
    
    const rect = canvasRef.current.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const t = Date.now();
    
    setTrajectory(prev => [...prev, { x, y, t }]);
  };

  const handleMouseUp = () => {
    if (isRecording && trajectory.length > 1) {
      const compressed = compressorRef.current.compress(trajectory, 'demo-node');
      setCompressedTrajectory(compressed);
    }
    setIsRecording(false);
  };

  const clearTrajectory = () => {
    setTrajectory([]);
    setCompressedTrajectory(null);
    setIsRecording(false);
  };

  const drawTrajectory = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Draw grid
    drawGrid(ctx, canvas.width, canvas.height);

    // Draw original trajectory
    if (trajectory.length > 1) {
      drawPath(ctx, trajectory, '#666', 1, 'Original Path');
    }

    // Draw compressed trajectory
    if (compressedTrajectory) {
      drawCompressedTrajectory(ctx, compressedTrajectory);
    }

    // Draw trajectory points
    trajectory.forEach((point, index) => {
      ctx.fillStyle = index === 0 ? '#4CAF50' : index === trajectory.length - 1 ? '#F44336' : '#2196F3';
      ctx.beginPath();
      ctx.arc(point.x, point.y, 3, 0, 2 * Math.PI);
      ctx.fill();
    });
  };

  const drawGrid = (ctx: CanvasRenderingContext2D, width: number, height: number) => {
    ctx.strokeStyle = '#f0f0f0';
    ctx.lineWidth = 0.5;
    
    for (let x = 0; x <= width; x += 20) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, height);
      ctx.stroke();
    }
    
    for (let y = 0; y <= height; y += 20) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
  };

  const drawPath = (ctx: CanvasRenderingContext2D, points: Point2D[], color: string, lineWidth: number, label: string) => {
    if (points.length < 2) return;

    ctx.strokeStyle = color;
    ctx.lineWidth = lineWidth;
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    
    for (let i = 1; i < points.length; i++) {
      ctx.lineTo(points[i].x, points[i].y);
    }
    
    ctx.stroke();

    // Draw label
    ctx.fillStyle = color;
    ctx.font = '12px Arial';
    ctx.fillText(label, points[0].x + 5, points[0].y - 5);
  };

  const drawCompressedTrajectory = (ctx: CanvasRenderingContext2D, compressed: CompressedTrajectory) => {
    compressed.segments.forEach((segment, index) => {
      const color = segment.type === 'linear' ? '#FF9800' : segment.type === 'bezier' ? '#9C27B0' : '#607D8B';
      
      switch (segment.type) {
        case 'linear':
          drawLinearSegment(ctx, segment, color, index);
          break;
        case 'bezier':
          drawBezierSegment(ctx, segment, color, index);
          break;
        case 'discrete':
          drawPath(ctx, segment.points, color, 2, `Discrete ${index + 1}`);
          break;
      }
    });
  };

  const drawLinearSegment = (ctx: CanvasRenderingContext2D, segment: any, color: string, index: number) => {
    ctx.strokeStyle = color;
    ctx.lineWidth = 3;
    ctx.setLineDash([5, 5]);
    
    ctx.beginPath();
    ctx.moveTo(segment.startPoint.x, segment.startPoint.y);
    
    // Calculate end point using velocity
    const duration = segment.endTime - segment.startPoint.t;
    const endX = segment.startPoint.x + segment.velocity.x * duration;
    const endY = segment.startPoint.y + segment.velocity.y * duration;
    
    ctx.lineTo(endX, endY);
    ctx.stroke();
    ctx.setLineDash([]);

    // Draw label
    ctx.fillStyle = color;
    ctx.font = '10px Arial';
    ctx.fillText(`Linear ${index + 1}`, segment.startPoint.x + 5, segment.startPoint.y - 10);
  };

  const drawBezierSegment = (ctx: CanvasRenderingContext2D, segment: any, color: string, index: number) => {
    const [p0, p1, p2, p3] = segment.controlPoints;
    
    ctx.strokeStyle = color;
    ctx.lineWidth = 3;
    ctx.setLineDash([10, 5]);
    
    ctx.beginPath();
    ctx.moveTo(p0.x, p0.y);
    ctx.bezierCurveTo(p1.x, p1.y, p2.x, p2.y, p3.x, p3.y);
    ctx.stroke();
    ctx.setLineDash([]);

    // Draw control points
    ctx.fillStyle = color;
    [p1, p2].forEach(cp => {
      ctx.beginPath();
      ctx.arc(cp.x, cp.y, 2, 0, 2 * Math.PI);
      ctx.fill();
    });

    // Draw control lines
    ctx.strokeStyle = color;
    ctx.lineWidth = 1;
    ctx.setLineDash([2, 2]);
    ctx.beginPath();
    ctx.moveTo(p0.x, p0.y);
    ctx.lineTo(p1.x, p1.y);
    ctx.moveTo(p2.x, p2.y);
    ctx.lineTo(p3.x, p3.y);
    ctx.stroke();
    ctx.setLineDash([]);

    // Draw label
    ctx.fillStyle = color;
    ctx.font = '10px Arial';
    ctx.fillText(`Bézier ${index + 1}`, p0.x + 5, p0.y - 10);
  };

  return (
    <div className={`trajectory-demo ${className || ''}`}>
      <div className="demo-header">
        <h3>轨迹压缩演示</h3>
        <div className="controls">
          <label>
            误差容差 (ε): 
            <input
              type="range"
              min="0.5"
              max="10"
              step="0.5"
              value={epsilon}
              onChange={(e) => setEpsilon(Number(e.target.value))}
            />
            <span>{epsilon}px</span>
          </label>
          <button onClick={clearTrajectory}>清除</button>
        </div>
      </div>
      
      <canvas
        ref={canvasRef}
        width={800}
        height={400}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        style={{
          border: '1px solid #ccc',
          cursor: isRecording ? 'crosshair' : 'pointer',
          backgroundColor: '#fafafa'
        }}
      />
      
      {compressedTrajectory && (
        <div className="trajectory-stats">
          <h4>压缩统计</h4>
          <div className="stats-grid">
            <div>原始点数: {trajectory.length}</div>
            <div>压缩段数: {compressedTrajectory.segments.length}</div>
            <div>压缩比: {(compressedTrajectory.compressionRatio * 100).toFixed(1)}%</div>
            <div>总误差: {compressedTrajectory.totalError.toFixed(2)}px</div>
          </div>
          
          <div className="segments-info">
            <h5>段信息:</h5>
            {compressedTrajectory.segments.map((segment, index) => (
              <div key={index} className="segment-info">
                <span className={`segment-type ${segment.type}`}>
                  {segment.type === 'linear' && '直线段'}
                  {segment.type === 'bezier' && '贝塞尔段'}
                  {segment.type === 'discrete' && '离散段'}
                </span>
                {segment.type === 'linear' && (
                  <span>速度: ({(segment as any).velocity.x.toFixed(2)}, {(segment as any).velocity.y.toFixed(2)})</span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
      
      <div className="demo-instructions">
        <h4>使用说明:</h4>
        <ul>
          <li>在画布上按住鼠标左键并拖动来绘制轨迹</li>
          <li>释放鼠标后会自动进行轨迹压缩</li>
          <li>调整误差容差来控制压缩精度</li>
          <li>颜色说明: 灰线=原轨迹, 橙线=直线段, 紫线=贝塞尔段</li>
        </ul>
      </div>

      <style jsx>{`
        .trajectory-demo {
          padding: 20px;
          max-width: 900px;
        }
        
        .demo-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
        }
        
        .controls {
          display: flex;
          align-items: center;
          gap: 20px;
        }
        
        .controls label {
          display: flex;
          align-items: center;
          gap: 10px;
        }
        
        .controls input[type="range"] {
          width: 120px;
        }
        
        .controls button {
          padding: 8px 16px;
          background: #f44336;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }
        
        .trajectory-stats {
          margin-top: 20px;
          padding: 15px;
          background: #f5f5f5;
          border-radius: 8px;
        }
        
        .stats-grid {
          display: grid;
          grid-template-columns: repeat(2, 1fr);
          gap: 10px;
          margin-bottom: 15px;
        }
        
        .segments-info {
          margin-top: 15px;
        }
        
        .segment-info {
          padding: 5px 10px;
          margin: 5px 0;
          background: white;
          border-radius: 4px;
          display: flex;
          justify-content: space-between;
        }
        
        .segment-type {
          font-weight: bold;
        }
        
        .segment-type.linear {
          color: #FF9800;
        }
        
        .segment-type.bezier {
          color: #9C27B0;
        }
        
        .segment-type.discrete {
          color: #607D8B;
        }
        
        .demo-instructions {
          margin-top: 20px;
          padding: 15px;
          background: #e3f2fd;
          border-radius: 8px;
        }
        
        .demo-instructions ul {
          margin: 10px 0;
          padding-left: 20px;
        }
        
        .demo-instructions li {
          margin: 5px 0;
        }
      `}</style>
    </div>
  );
};

export default TrajectoryDemo; 