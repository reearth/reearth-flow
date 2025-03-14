import { EdgeChange } from "@xyflow/react";
import { Dispatch, SetStateAction, useCallback } from "react";

import { Edge } from "@flow/types";

import { yEdgeConstructor } from "./conversions";
import type { YEdgesArray, YWorkflow } from "./types";

export default ({
  currentYWorkflow,
  setSelectedEdgeIds,
  undoTrackerActionWrapper,
}: {
  currentYWorkflow?: YWorkflow;
  setSelectedEdgeIds: Dispatch<SetStateAction<string[]>>;
  undoTrackerActionWrapper: (callback: () => void) => void;
}) => {
  const handleYEdgesAdd = useCallback(
    (newEdges: Edge[]) => {
      undoTrackerActionWrapper(() => {
        const yEdges = currentYWorkflow?.get("edges") as
          | YEdgesArray
          | undefined;
        if (!yEdges) return;
        const newYEdges = newEdges.map((newEdge) => yEdgeConstructor(newEdge));

        newEdges.forEach((newEdge) => {
          if (newEdge.selected) {
            setSelectedEdgeIds((seids) => {
              return [...seids, newEdge.id];
            });
          }
        });

        yEdges.insert(yEdges.length, newYEdges);
      });
    },
    [currentYWorkflow, setSelectedEdgeIds, undoTrackerActionWrapper],
  );

  const handleYEdgesChange = useCallback(
    (changes: EdgeChange[]) => {
      const yEdges = currentYWorkflow?.get("edges") as YEdgesArray | undefined;
      if (!yEdges) return;

      const existingEdgesMap = new Map(
        Array.from(yEdges).map((yEdge, index) => [
          yEdge.get("id")?.toString(),
          { yEdge, index },
        ]),
      );

      undoTrackerActionWrapper(() => {
        changes.forEach((change) => {
          switch (change.type) {
            case "add": {
              const newYEdge = yEdgeConstructor(change.item);
              yEdges.insert(yEdges.length, [newYEdge]);
              break;
            }
            case "replace": {
              const existing = existingEdgesMap.get(change.id);

              if (existing) {
                const index = Array.from(yEdges).findIndex(
                  (ye) => ye.get("id")?.toString() === change.id,
                );

                if (index !== -1) {
                  setSelectedEdgeIds((seids) => {
                    return seids.filter((seid) => seid !== change.id);
                  });

                  const newYEdge = yEdgeConstructor(change.item);
                  yEdges.delete(index, 1);
                  yEdges.insert(index, [newYEdge]);
                }
              }
              break;
            }
            case "remove": {
              const existing = existingEdgesMap.get(change.id);

              if (existing) {
                const index = Array.from(yEdges).findIndex(
                  (yn) => yn.get("id")?.toString() === change.id,
                );

                if (index !== -1) {
                  setSelectedEdgeIds((seids) => {
                    return seids.filter((seid) => seid !== change.id);
                  });

                  yEdges.delete(index, 1);
                }
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
