import type { NodeChange } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback, useRef } from "react";
import * as Y from "yjs";

import { TrajectoryCompressor, Point2D, CompressedTrajectory } from "@flow/lib/trajectory";
import type { Node, Workflow } from "@flow/types";

import { yNodeConstructor } from "./conversions";
import type { YWorkflow,  YNodesMap, YNodeValue } from "./types";
import { updateParentYWorkflow } from "./useParentYWorkflow";
import { removeParentYWorkflowNodePseudoPort } from "./useParentYWorkflow/removeParentYWorkflowNodePseudoPort";

// Trajectory compression state
const trajectoryCompressor = new TrajectoryCompressor(1.0); // 1 pixel error tolerance
const nodeTrajectories = new Map<string, Point2D[]>();
const compressedTrajectories = new Map<string, CompressedTrajectory>();
const MAX_TRAJECTORY_POINTS = 50; // Compress when reaching this many points
const UPDATE_THROTTLE_MS = 50; // Shorter throttle for better real-time experience
const IMMEDIATE_UPDATE_DISTANCE = 10; // Send immediate update if moved more than 10 pixels
const pendingUpdates = new Map<string, { x: number; y: number; timestamp: number; lastSentPosition?: { x: number; y: number } }>();
const updateTimers = new Map<string, NodeJS.Timeout>();

export default ({
  currentYWorkflow,
  yWorkflows,
  rawWorkflows,
  setSelectedNodeIds,
  undoTrackerActionWrapper,
  handleYWorkflowRemove,
}: {
  currentYWorkflow?: YWorkflow;
  yWorkflows: Y.Map<YWorkflow>;
  rawWorkflows: Workflow[];
  setSelectedNodeIds: Dispatch<SetStateAction<string[]>>;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
  ) => void;
  handleYWorkflowRemove?: (workflowId: string) => void;
}) => {
  const handleYNodesAdd = useCallback(
    (newNodes: Node[]) => {
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow?.get("nodes") as YNodesMap | undefined;
        if (!yNodes) return;

        newNodes.forEach((newNode) => {
          if (newNode.selected) {
            setSelectedNodeIds((snids) => {
              return [...snids, newNode.id];
            });
          }
          yNodes.set(newNode.id, yNodeConstructor(newNode));
        });
      });
    },
    [currentYWorkflow, setSelectedNodeIds, undoTrackerActionWrapper],
  );

  // Process node position update with trajectory compression and smart update strategy
  const processNodePositionUpdate = useCallback((nodeId: string, x: number, y: number): 'immediate' | 'compressed' | 'throttled' => {
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
      
      console.log(`Compressed trajectory for node ${nodeId}, compression ratio: ${compressed.compressionRatio.toFixed(2)}`);
      
      return 'compressed';
    }
    
    // Return immediate for significant movements, otherwise throttled
    return shouldSendImmediate ? 'immediate' : 'throttled';
  }, []);

  // Apply immediate position update to Yjs
  const applyImmediateUpdate = useCallback((nodeId: string, x: number, y: number, yNodes: YNodesMap) => {
    const existingYNode = yNodes.get(nodeId);
    if (existingYNode) {
      const newPosition = new Y.Map<unknown>();
      newPosition.set("x", x);
      newPosition.set("y", y);
      existingYNode.set("position", newPosition);
    }
  }, []);

  // Apply compressed trajectory updates to Yjs
  const applyCompressedUpdate = useCallback((nodeId: string, yNodes: YNodesMap) => {
    const compressed = compressedTrajectories.get(nodeId);
    if (!compressed) return;

    // Get the latest position from compressed trajectory
    const latestTime = Date.now();
    const position = trajectoryCompressor.getPositionAtTime(compressed, latestTime);
    
    if (position) {
      const existingYNode = yNodes.get(nodeId);
      if (existingYNode) {
        const newPosition = new Y.Map<unknown>();
        newPosition.set("x", position.x);
        newPosition.set("y", position.y);
        existingYNode.set("position", newPosition);
      }
    }
  }, []);

  // Throttled update using pending positions
  const scheduleThrottledUpdate = useCallback((nodeId: string, yNodes: YNodesMap) => {
    // Clear existing timer
    const existingTimer = updateTimers.get(nodeId);
    if (existingTimer) {
      clearTimeout(existingTimer);
    }

    // Schedule new update
    const timer = setTimeout(() => {
      const pendingUpdate = pendingUpdates.get(nodeId);
      if (pendingUpdate) {
        const existingYNode = yNodes.get(nodeId);
        if (existingYNode) {
          const newPosition = new Y.Map<unknown>();
          newPosition.set("x", pendingUpdate.x);
          newPosition.set("y", pendingUpdate.y);
          existingYNode.set("position", newPosition);
        }
        pendingUpdates.delete(nodeId);
      }
      updateTimers.delete(nodeId);
    }, UPDATE_THROTTLE_MS);

    updateTimers.set(nodeId, timer);
  }, []);

  // Force flush all pending updates (useful when drag ends)
  const flushPendingUpdates = useCallback((yNodes: YNodesMap) => {
    pendingUpdates.forEach((update, nodeId) => {
      const existingYNode = yNodes.get(nodeId);
      if (existingYNode) {
        const newPosition = new Y.Map<unknown>();
        newPosition.set("x", update.x);
        newPosition.set("y", update.y);
        existingYNode.set("position", newPosition);
      }
    });
    
    // Clear all pending updates and timers
    pendingUpdates.clear();
    updateTimers.forEach(timer => clearTimeout(timer));
    updateTimers.clear();
  }, []);

  // Get interpolated position for smooth animation
  const getInterpolatedPosition = useCallback((nodeId: string, timestamp: number): Point2D | null => {
    // First check compressed trajectories
    const compressed = compressedTrajectories.get(nodeId);
    if (compressed) {
      const position = trajectoryCompressor.getPositionAtTime(compressed, timestamp);
      if (position) return position;
    }
    
    // Fallback to raw trajectory points
    const trajectory = nodeTrajectories.get(nodeId);
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
  }, []);

  // Passed to editor context so needs to be a ref
  const handleYNodesChangeRef =
    useRef<(changes: NodeChange[]) => void>(undefined);
  // This is based off of react-flow node changes, which includes removal
  // but not addtion. This is why we have a separate function for adding nodes.
  handleYNodesChangeRef.current = (changes: NodeChange[]) => {
    const yNodes = currentYWorkflow?.get("nodes") as YNodesMap | undefined;
    if (!yNodes) return;

    undoTrackerActionWrapper(() => {
      changes.forEach((change) => {
        switch (change.type) {
          case "position": {
            const existingYNode = yNodes.get(change.id);

            if (existingYNode && change.position) {
              // Process trajectory compression and determine update strategy
              const updateStrategy = processNodePositionUpdate(change.id, change.position.x, change.position.y);
              
              switch (updateStrategy) {
                case 'immediate':
                  // Send immediately for better real-time collaboration
                  applyImmediateUpdate(change.id, change.position.x, change.position.y, yNodes);
                  break;
                case 'compressed':
                  // Use compressed trajectory for position update
                  applyCompressedUpdate(change.id, yNodes);
                  break;
                case 'throttled':
                  // Use throttled updates for performance optimization
                  scheduleThrottledUpdate(change.id, yNodes);
                  break;
              }
            }
            break;
          }
          case "replace": {
            const existingYNode = yNodes.get(change.id);

            if (existingYNode && change.item) {
              const newYNode = yNodeConstructor(change.item as Node);
              yNodes.set(change.id, newYNode);
            }
            break;
          }
          case "dimensions": {
            const existingYNode = yNodes.get(change.id);

            if (existingYNode && change.dimensions) {
              const newMeasured = new Y.Map<unknown>();
              newMeasured.set("width", change.dimensions.width);
              newMeasured.set("height", change.dimensions.height);
              existingYNode?.set("measured", newMeasured);

              if (change.setAttributes) {
                const newStyle = new Y.Map<unknown>();
                newStyle.set("width", change.dimensions.width + "px");
                newStyle.set("height", change.dimensions.height + "px");
                existingYNode?.set("style", newStyle);
              }
            }
            break;
          }
          case "remove": {
            const existingYNode = yNodes.get(change.id);

            if (existingYNode) {
              const nodeToDelete = existingYNode.toJSON() as Node;
              
              // Clean up trajectory data for removed node
              nodeTrajectories.delete(change.id);
              compressedTrajectories.delete(change.id);
              pendingUpdates.delete(change.id);
              
              // Clear any pending update timers
              const timer = updateTimers.get(change.id);
              if (timer) {
                clearTimeout(timer);
                updateTimers.delete(change.id);
              }
              
              if (
                nodeToDelete.type === "subworkflow" &&
                nodeToDelete.data.subworkflowId
              ) {
                handleYWorkflowRemove?.(nodeToDelete.data.subworkflowId);
              } else if (nodeToDelete.data.params?.routingPort) {
                const parentWorkflowId = rawWorkflows.find((w) => {
                  const nodes = w.nodes as Node[];
                  return nodes.some(
                    (n) =>
                      n.id ===
                      (currentYWorkflow?.get("id")?.toJSON() as string),
                  );
                })?.id;
                if (!parentWorkflowId) return;
                const parentYWorkflow = yWorkflows.get(parentWorkflowId);
                if (parentYWorkflow) {
                  removeParentYWorkflowNodePseudoPort(
                    currentYWorkflow?.get("id")?.toJSON() as string,
                    parentYWorkflow,
                    nodeToDelete,
                  );
                }
              }

              setSelectedNodeIds((snids) => {
                return snids.filter((snid) => snid !== change.id);
              });

              yNodes.delete(change.id);
            }
            break;
          }
          case "select": {
            setSelectedNodeIds((snids) => {
              if (change.selected) {
                return [...snids, change.id];
              } else {
                return snids.filter((snid) => snid !== change.id);
              }
            });
            break;
          }
        }
      });
    });
  };
  const handleYNodesChange = useCallback(
    (changes: NodeChange[]) => handleYNodesChangeRef.current?.(changes),
    [],
  );

  const handleYNodeDataUpdate = useCallback(
    (
      nodeId: string,
      dataField: "params" | "customizations",
      updatedValue: any,
    ) =>
      undoTrackerActionWrapper(() => {
        const yNodes = currentYWorkflow?.get("nodes") as YNodesMap | undefined;
        if (!yNodes) return;

        const nodes = Object.values(yNodes.toJSON()) as Node[];

        const prevNode = nodes.find((n) => n.id === nodeId);

        if (!prevNode) return;
        // if params.routingPort exists, it's parent is a subworkflow and
        // we need to update pseudoInputs and pseudoOutputs on the parent node.
        if (dataField === "params" && updatedValue.routingPort) {
          const currentWorkflowId = currentYWorkflow
            ?.get("id")
            ?.toJSON() as string;

          const parentWorkflow = rawWorkflows.find((w) => {
            const nodes = w.nodes as Node[];
            return nodes.some(
              (n) => n.data.subworkflowId === currentWorkflowId,
            );
          });
          if (!parentWorkflow) return;
          const parentYWorkflow = yWorkflows.get(parentWorkflow.id);
          if (!parentYWorkflow) return;

          updateParentYWorkflow(
            currentWorkflowId,
            parentYWorkflow,
            prevNode,
            updatedValue,
          );
        }

        const yData = yNodes.get(nodeId)?.get("data") as Y.Map<YNodeValue>;
        yData?.set(dataField, updatedValue);
      }),
    [currentYWorkflow, rawWorkflows, yWorkflows, undoTrackerActionWrapper],
  );

  return {
    handleYNodesAdd,
    handleYNodesChange,
    handleYNodeDataUpdate,
    // Expose trajectory utilities for external use
    getInterpolatedPosition,
    getCompressedTrajectory: (nodeId: string) => compressedTrajectories.get(nodeId),
    clearTrajectoryData: (nodeId: string) => {
      nodeTrajectories.delete(nodeId);
      compressedTrajectories.delete(nodeId);
      pendingUpdates.delete(nodeId);
      
      // Clear any pending update timers
      const timer = updateTimers.get(nodeId);
      if (timer) {
        clearTimeout(timer);
        updateTimers.delete(nodeId);
      }
    },
    // Force flush pending updates when drag operation ends
    flushPendingUpdates: () => {
      const yNodes = currentYWorkflow?.get("nodes") as YNodesMap | undefined;
      if (yNodes) {
        flushPendingUpdates(yNodes);
      }
    },
  };
};
