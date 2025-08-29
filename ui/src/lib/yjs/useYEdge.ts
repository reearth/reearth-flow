import { EdgeChange } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback } from "react";

import { Edge } from "@flow/types";

import { yEdgeConstructor } from "./conversions";
import type { YEdge, YEdgesMap, YWorkflow } from "./types";

export default ({
  currentYWorkflow,
  setSelectedEdgeIds,
  undoTrackerActionWrapper,
}: {
  currentYWorkflow?: YWorkflow;
  setSelectedEdgeIds: Dispatch<SetStateAction<string[]>>;
  undoTrackerActionWrapper: (
    callback: () => void,
    originPrepend?: string,
    throttleMs?: number,
  ) => void;
}) => {
  const handleYEdgesAdd = useCallback(
    (newEdges: Edge[]) => {
      undoTrackerActionWrapper(() => {
        const yEdges = currentYWorkflow?.get("edges") as YEdgesMap | undefined;
        if (!yEdges) return;
        const newYEdges = new Map<string, YEdge>();
        newEdges.forEach((newEdge) => {
          const newYEdge = yEdgeConstructor(newEdge);
          newYEdges.set(newEdge.id, newYEdge);
        });

        newEdges.forEach((newEdge) => {
          if (newEdge.selected) {
            setSelectedEdgeIds((seids) => {
              return [...seids, newEdge.id];
            });
          }
        });
        newYEdges.forEach((newYEdge, key) => {
          yEdges.set(key, newYEdge);
        });
      });
    },
    [currentYWorkflow, setSelectedEdgeIds, undoTrackerActionWrapper],
  );

  const handleYEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      const yEdges = currentYWorkflow?.get("edges") as YEdgesMap | undefined;
      if (!yEdges) return;

      undoTrackerActionWrapper(() => {
        changes.forEach((change) => {
          switch (change.type) {
            case "add": {
              const newYEdge = yEdgeConstructor(change.item);
              yEdges.set(change.item.id, newYEdge);
              break;
            }
            case "replace": {
              const existing = yEdges.get(change.id);

              if (existing) {
                setSelectedEdgeIds((seids) => {
                  return seids.filter((seid) => seid !== change.id);
                });

                const newYEdge = yEdgeConstructor(change.item);
                yEdges.set(change.id, newYEdge);
              }
              break;
            }
            case "remove": {
              const existing = yEdges.get(change.id);

              if (existing) {
                setSelectedEdgeIds((seids) => {
                  return seids.filter((seid) => seid !== change.id);
                });

                yEdges.delete(change.id);
              }
              break;
            }
            case "select": {
              setSelectedEdgeIds((seids) => {
                if (change.selected) {
                  return [...seids, change.id];
                } else {
                  return seids.filter((seid) => seid !== change.id);
                }
              });
              break;
            }
          }
        });
      });
    },
    [currentYWorkflow, setSelectedEdgeIds, undoTrackerActionWrapper],
  );
  return {
    handleYEdgesAdd,
    handleYEdgesChange,
  };
};
