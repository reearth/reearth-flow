import { useCallback, useEffect, useRef } from 'react';
import { TrajectoryCompressor, Point2D, CompressedTrajectory } from './index';

interface TrajectoryState {
  compressor: TrajectoryCompressor;
  nodeTrajectories: Map<string, Point2D[]>;
  compressedTrajectories: Map<string, CompressedTrajectory>;
  animationFrames: Map<string, number>;
}

interface UseTrajectorySmoothinOptions {
  epsilon?: number; // Error tolerance in pixels
  maxTrajectoryPoints?: number; // Max points before compression
  smoothingDuration?: number; // Duration for smooth transitions in ms
  enableSmoothing?: boolean; // Enable/disable smoothing
}

/**
 * Hook for managing trajectory compression and smooth node movement
 */
export const useTrajectorySmoothing = (options: UseTrajectorySmoothinOptions = {}) => {
  const {
    epsilon = 1.0,
    maxTrajectoryPoints = 50,
    smoothingDuration = 300,
    enableSmoothing = true,
  } = options;

  const stateRef = useRef<TrajectoryState>({
    compressor: new TrajectoryCompressor(epsilon),
    nodeTrajectories: new Map(),
    compressedTrajectories: new Map(),
    animationFrames: new Map(),
  });

  const state = stateRef.current;

  /**
   * Add a position update to a node's trajectory
   */
  const addPositionUpdate = useCallback((nodeId: string, x: number, y: number, timestamp?: number) => {
    const t = timestamp ?? Date.now();
    const newPoint: Point2D = { x, y, t };
    
    // Get or create trajectory for this node
    let trajectory = state.nodeTrajectories.get(nodeId);
    if (!trajectory) {
      trajectory = [];
      state.nodeTrajectories.set(nodeId, trajectory);
    }
    
    trajectory.push(newPoint);
    
    // Compress trajectory when it gets too long
    if (trajectory.length >= maxTrajectoryPoints) {
      const compressed = state.compressor.compress([...trajectory], nodeId);
      state.compressedTrajectories.set(nodeId, compressed);
      
      // Keep only recent points for future compression
      const keepRecentPoints = 10;
      trajectory.splice(0, trajectory.length - keepRecentPoints);
      
      console.log(`Trajectory compressed for node ${nodeId}, ratio: ${compressed.compressionRatio.toFixed(2)}`);
    }
  }, [maxTrajectoryPoints, state]);

  /**
   * Get smoothed position for a node at a specific time
   */
  const getSmoothedPosition = useCallback((nodeId: string, timestamp: number): Point2D | null => {
    if (!enableSmoothing) return null;

    // First check compressed trajectories
    const compressed = state.compressedTrajectories.get(nodeId);
    if (compressed) {
      const position = state.compressor.getPositionAtTime(compressed, timestamp);
      if (position) return position;
    }
    
    // Fallback to raw trajectory points
    const trajectory = state.nodeTrajectories.get(nodeId);
    if (!trajectory || trajectory.length === 0) return null;
    
    // Linear interpolation between nearest points
    for (let i = 0; i < trajectory.length - 1; i++) {
      if (timestamp >= trajectory[i].t && timestamp <= trajectory[i + 1].t) {
        const dt = trajectory[i + 1].t - trajectory[i].t;
        if (dt > 1e-10) {
          const alpha = (timestamp - trajectory[i].t) / dt;
          return {
            x: trajectory[i].x + alpha * (trajectory[i + 1].x - trajectory[i].x),
            y: trajectory[i].y + alpha * (trajectory[i + 1].y - trajectory[i].y),
            t: timestamp,
          };
        } else {
          return trajectory[i];
        }
      }
    }
    
    // Return closest point if timestamp is outside range
    if (timestamp <= trajectory[0].t) {
      return trajectory[0];
    } else {
      return trajectory[trajectory.length - 1];
    }
  }, [enableSmoothing, state]);

  /**
   * Animate smooth movement from current position to target position
   */
  const animateToPosition = useCallback((
    nodeId: string,
    fromPosition: { x: number; y: number },
    toPosition: { x: number; y: number },
    onUpdate: (position: { x: number; y: number }) => void,
    duration: number = smoothingDuration
  ) => {
    if (!enableSmoothing) {
      onUpdate(toPosition);
      return;
    }

    // Cancel any existing animation for this node
    const existingFrame = state.animationFrames.get(nodeId);
    if (existingFrame) {
      cancelAnimationFrame(existingFrame);
    }

    const startTime = Date.now();
    const deltaX = toPosition.x - fromPosition.x;
    const deltaY = toPosition.y - fromPosition.y;

    const animate = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);
      
      // Use easing function for smooth animation
      const eased = easeOutCubic(progress);
      
      const currentPosition = {
        x: fromPosition.x + deltaX * eased,
        y: fromPosition.y + deltaY * eased,
      };

      onUpdate(currentPosition);

      if (progress < 1) {
        const frameId = requestAnimationFrame(animate);
        state.animationFrames.set(nodeId, frameId);
      } else {
        state.animationFrames.delete(nodeId);
      }
    };

    const frameId = requestAnimationFrame(animate);
    state.animationFrames.set(nodeId, frameId);
  }, [enableSmoothing, smoothingDuration, state]);

  /**
   * Get compressed trajectory for a node
   */
  const getCompressedTrajectory = useCallback((nodeId: string): CompressedTrajectory | undefined => {
    return state.compressedTrajectories.get(nodeId);
  }, [state]);

  /**
   * Clear trajectory data for a node
   */
  const clearTrajectoryData = useCallback((nodeId: string) => {
    state.nodeTrajectories.delete(nodeId);
    state.compressedTrajectories.delete(nodeId);
    
    // Cancel any ongoing animation
    const frameId = state.animationFrames.get(nodeId);
    if (frameId) {
      cancelAnimationFrame(frameId);
      state.animationFrames.delete(nodeId);
    }
  }, [state]);

  /**
   * Get trajectory statistics
   */
  const getTrajectoryStats = useCallback(() => {
    const totalNodes = state.nodeTrajectories.size;
    const compressedNodes = state.compressedTrajectories.size;
    let totalPoints = 0;
    let totalCompressedSegments = 0;
    
    state.nodeTrajectories.forEach(trajectory => {
      totalPoints += trajectory.length;
    });
    
    state.compressedTrajectories.forEach(compressed => {
      totalCompressedSegments += compressed.segments.length;
    });
    
    const averageCompressionRatio = compressedNodes > 0 
      ? Array.from(state.compressedTrajectories.values())
          .reduce((sum, trajectory) => sum + trajectory.compressionRatio, 0) / compressedNodes
      : 0;

    return {
      totalNodes,
      compressedNodes,
      totalPoints,
      totalCompressedSegments,
      averageCompressionRatio,
    };
  }, [state]);

  /**
   * Cleanup on unmount
   */
  useEffect(() => {
    return () => {
      // Cancel all ongoing animations
      state.animationFrames.forEach(frameId => {
        cancelAnimationFrame(frameId);
      });
      state.animationFrames.clear();
    };
  }, [state]);

  return {
    addPositionUpdate,
    getSmoothedPosition,
    animateToPosition,
    getCompressedTrajectory,
    clearTrajectoryData,
    getTrajectoryStats,
  };
};

/**
 * Easing function for smooth animations
 */
function easeOutCubic(t: number): number {
  return 1 - Math.pow(1 - t, 3);
}

export default useTrajectorySmoothing; 