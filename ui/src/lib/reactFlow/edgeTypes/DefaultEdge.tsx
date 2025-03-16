import { Table } from "@phosphor-icons/react";
import {
  BaseEdge,
  EdgeLabelRenderer,
  EdgeProps,
  getBezierPath,
} from "@xyflow/react";
import { useCallback } from "react";

import { useIndexedDB } from "@flow/lib/indexedDB";
import { DebugRunState, useCurrentProject } from "@flow/stores";
import { Edge } from "@flow/types";

export type CustomEdgeProps = EdgeProps<Edge>;

const DefaultEdge: React.FC<CustomEdgeProps> = ({
  id,
  sourceX,
  sourceY,
  sourcePosition,
  targetX,
  targetY,
  targetPosition,
  selected,
  // markerEnd,
  // ...props
}) => {
  const [currentProject] = useCurrentProject();

  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const { value: debugRunState, updateValue } = useIndexedDB("debugRun");

  const handleIntermediateDataSet = useCallback(() => {
    if (!selected) return;
    const newDebugRunState: DebugRunState = {
      ...debugRunState,
      jobs:
        debugRunState?.jobs?.map((job) =>
          job.projectId === currentProject?.id
            ? {
                ...job,
                selectedIntermediateData: {
                  edgeId: id,
                  url: "/7571eea0-eabf-4ff7-b978-e5965d882409.jsonl", //TODO: replace with actual intermediate data
                },
              }
            : job,
        ) ?? [],
    };
    updateValue(newDebugRunState);
  }, [selected, debugRunState, currentProject, id, updateValue]);

  return (
    <>
      <BaseEdge id={id} path={edgePath} />
      <EdgeLabelRenderer>
        <Table
          className="nodrag nopan absolute size-[30px] origin-center rounded-full border border-white bg-primary p-1 transition-[height,width] hover:size-[50px]"
          style={{
            pointerEvents: "all",
            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
          }}
          weight="bold"
          onDoubleClick={handleIntermediateDataSet}
        />
      </EdgeLabelRenderer>
      {/* {nodeRunning && (
        <>
        <path
        d={edgePath}
        stroke="#27272A"
        strokeWidth="2"
        strokeDasharray="20,20"
        fill="none">
        <animate
        attributeName="stroke-dashoffset"
        from="40"
        to="0"
        dur="1s"
        repeatCount="indefinite"
        />
        </path>
        <g>
        <circle className="opacity-25" r="8" fill="#ffffff">
        <animateMotion
        dur="5s"
        repeatCount="indefinite"
        path={edgePath}
        />
        </circle>
        <circle
        style={{ filter: `drop-shadow(3px 3px 5px #471a27)` }}
        r="3"
        fill="#bbffff">
        <animateMotion
        dur="5s"
        repeatCount="indefinite"
        path={edgePath}
        />
        </circle>
        </g>
        </>
        )} */}
    </>
  );
};

export default DefaultEdge;
