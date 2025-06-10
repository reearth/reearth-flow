import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';

// Mock Yjs for testing
const mockYMapInstance = {
  set: vi.fn(),
  get: vi.fn(),
  delete: vi.fn(),
  toJSON: vi.fn(() => ({})),
};

const mockYMap = vi.fn(() => mockYMapInstance);

vi.mock('yjs', () => ({
  Map: mockYMap,
}));

// Import the hook components we need to test
import { TrajectoryCompressor, Point2D } from './index';

describe('Real-time Update Strategy', () => {
  let trajectoryCompressor: TrajectoryCompressor;
  let nodeTrajectories: Map<string, Point2D[]>;
  let compressedTrajectories: Map<string, any>;
  let pendingUpdates: Map<string, any>;
  let updateTimers: Map<string, NodeJS.Timeout>;

  const IMMEDIATE_UPDATE_DISTANCE = 10;
  const MAX_TRAJECTORY_POINTS = 50;

  beforeEach(() => {
    trajectoryCompressor = new TrajectoryCompressor(1.0);
    nodeTrajectories = new Map();
    compressedTrajectories = new Map();
    pendingUpdates = new Map();
    updateTimers = new Map();
    vi.clearAllMocks();
  });

  afterEach(() => {
    // Clear any pending timers
    updateTimers.forEach(timer => clearTimeout(timer));
    updateTimers.clear();
  });

  // Simulate the processNodePositionUpdate function logic
  function processNodePositionUpdate(nodeId: string, x: number, y: number): 'immediate' | 'compressed' | 'throttled' {
    const currentTime = Date.now();
    const newPoint: Point2D = { x, y, t: currentTime };
    
    // Get or create trajectory for this node
    let trajectory = nodeTrajectories.get(nodeId);
    if (!trajectory) {
      trajectory = [];
      nodeTrajectories.set(nodeId, trajectory);
    }
    
    trajectory.push(newPoint);
    
    // Get current pending update to check movement distance
    const currentPending = pendingUpdates.get(nodeId);
    const lastSentPos = currentPending?.lastSentPosition;
    
    // Calculate distance moved since last sent position
    let shouldSendImmediate = false;
    if (lastSentPos) {
      const distance = Math.sqrt(
        Math.pow(x - lastSentPos.x, 2) + Math.pow(y - lastSentPos.y, 2)
      );
      shouldSendImmediate = distance >= IMMEDIATE_UPDATE_DISTANCE;
    } else {
      // First movement always sends immediately for better responsiveness
      shouldSendImmediate = true;
    }
    
    // Store pending update for throttling
    pendingUpdates.set(nodeId, { 
      x, 
      y, 
      timestamp: currentTime,
      lastSentPosition: shouldSendImmediate ? { x, y } : lastSentPos
    });
    
    // Compress trajectory when it gets too long
    if (trajectory.length >= MAX_TRAJECTORY_POINTS) {
      const compressed = trajectoryCompressor.compress([...trajectory], nodeId);
      compressedTrajectories.set(nodeId, compressed);
      
      // Keep only recent points for future compression
      const keepRecentPoints = 10;  
      trajectory.splice(0, trajectory.length - keepRecentPoints);
      
      return 'compressed';
    }
    
    // Return immediate for significant movements, otherwise throttled
    return shouldSendImmediate ? 'immediate' : 'throttled';
  }

  describe('Update Strategy Selection', () => {
    it('should return immediate for first movement', () => {
      const strategy = processNodePositionUpdate('node1', 100, 100);
      expect(strategy).toBe('immediate');
    });

    it('should return throttled for small movements', () => {
      // First movement (immediate)
      processNodePositionUpdate('node1', 100, 100);
      
      // Small movement (should be throttled)
      const strategy = processNodePositionUpdate('node1', 105, 105);
      expect(strategy).toBe('throttled');
    });

    it('should return immediate for large movements', () => {
      // First movement (immediate)
      processNodePositionUpdate('node1', 100, 100);
      
      // Large movement (should be immediate)
      const strategy = processNodePositionUpdate('node1', 120, 120);
      expect(strategy).toBe('immediate');
    });

    it('should return compressed when trajectory gets too long', () => {
      const nodeId = 'node1';
      
      // Add many points to trigger compression
      for (let i = 0; i < MAX_TRAJECTORY_POINTS; i++) {
        const strategy = processNodePositionUpdate(nodeId, i, i);
        if (i === MAX_TRAJECTORY_POINTS - 1) {
          expect(strategy).toBe('compressed');
        }
      }
    });

    it('should track last sent position correctly', () => {
      const nodeId = 'node1';
      
      // First movement (immediate)
      processNodePositionUpdate(nodeId, 0, 0);
      let pending = pendingUpdates.get(nodeId);
      expect(pending?.lastSentPosition).toEqual({ x: 0, y: 0 });
      
      // Small movement (throttled, keeps last sent position)
      processNodePositionUpdate(nodeId, 5, 5);
      pending = pendingUpdates.get(nodeId);
      expect(pending?.lastSentPosition).toEqual({ x: 0, y: 0 });
      
      // Large movement (immediate, updates last sent position)
      processNodePositionUpdate(nodeId, 20, 20);
      pending = pendingUpdates.get(nodeId);
      expect(pending?.lastSentPosition).toEqual({ x: 20, y: 20 });
    });
  });

  describe('Distance Calculation', () => {
    it('should calculate distance correctly for diagonal movement', () => {
      const nodeId = 'node1';
      
      // First movement
      processNodePositionUpdate(nodeId, 0, 0);
      
      // Move diagonally - distance should be sqrt(8^2 + 6^2) = 10
      const strategy = processNodePositionUpdate(nodeId, 8, 6);
      expect(strategy).toBe('immediate'); // exactly at threshold
    });

    it('should handle edge case at distance threshold', () => {
      const nodeId = 'node1';
      
      // First movement
      processNodePositionUpdate(nodeId, 0, 0);
      
      // Move exactly IMMEDIATE_UPDATE_DISTANCE
      const strategy = processNodePositionUpdate(nodeId, IMMEDIATE_UPDATE_DISTANCE, 0);
      expect(strategy).toBe('immediate');
    });

    it('should handle very small movements', () => {
      const nodeId = 'node1';
      
      // First movement
      processNodePositionUpdate(nodeId, 0, 0);
      
      // Very small movement
      const strategy = processNodePositionUpdate(nodeId, 0.1, 0.1);
      expect(strategy).toBe('throttled');
    });
  });

  describe('Performance Characteristics', () => {
    it('should maintain reasonable trajectory size after compression', () => {
      const nodeId = 'node1';
      
      // Add many points to trigger compression
      for (let i = 0; i < MAX_TRAJECTORY_POINTS + 20; i++) {
        processNodePositionUpdate(nodeId, i, i);
      }
      
      const trajectory = nodeTrajectories.get(nodeId);
      expect(trajectory?.length).toBeLessThan(MAX_TRAJECTORY_POINTS);
      expect(compressedTrajectories.has(nodeId)).toBe(true);
    });

    it('should handle rapid position updates efficiently', () => {
      const nodeId = 'node1';
      const startTime = Date.now();
      
      // Simulate rapid updates
      for (let i = 0; i < 100; i++) {
        processNodePositionUpdate(nodeId, i * 2, i * 2);
      }
      
      const endTime = Date.now();
      expect(endTime - startTime).toBeLessThan(100); // Should complete in < 100ms
    });
  });

  describe('Multiple Nodes', () => {
    it('should handle multiple nodes independently', () => {
      const node1Strategy = processNodePositionUpdate('node1', 100, 100);
      const node2Strategy = processNodePositionUpdate('node2', 200, 200);
      
      expect(node1Strategy).toBe('immediate');
      expect(node2Strategy).toBe('immediate');
      
      expect(pendingUpdates.size).toBe(2);
      expect(nodeTrajectories.size).toBe(2);
    });

    it('should maintain separate trajectories for different nodes', () => {
      // Add different paths for different nodes
      processNodePositionUpdate('node1', 0, 0);
      processNodePositionUpdate('node2', 100, 100);
      processNodePositionUpdate('node1', 10, 10);
      processNodePositionUpdate('node2', 110, 110);
      
      const trajectory1 = nodeTrajectories.get('node1');
      const trajectory2 = nodeTrajectories.get('node2');
      
      expect(trajectory1?.length).toBe(2);
      expect(trajectory2?.length).toBe(2);
      expect(trajectory1?.[0]).toEqual({ x: 0, y: 0, t: expect.any(Number) });
      expect(trajectory2?.[0]).toEqual({ x: 100, y: 100, t: expect.any(Number) });
    });
  });

  describe('Edge Cases', () => {
    it('should handle zero movement', () => {
      const nodeId = 'node1';
      
      processNodePositionUpdate(nodeId, 100, 100);
      const strategy = processNodePositionUpdate(nodeId, 100, 100);
      
      expect(strategy).toBe('throttled'); // Same position = distance 0
    });

    it('should handle negative coordinates', () => {
      const nodeId = 'node1';
      
      processNodePositionUpdate(nodeId, 0, 0);
      const strategy = processNodePositionUpdate(nodeId, -15, -15);
      
      expect(strategy).toBe('immediate'); // Distance > threshold
    });

    it('should handle very large coordinates', () => {
      const nodeId = 'node1';
      
      const strategy = processNodePositionUpdate(nodeId, 1000000, 1000000);
      expect(strategy).toBe('immediate');
    });
  });
}); 