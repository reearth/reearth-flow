import { TableIcon, XIcon } from "@phosphor-icons/react";
import {
  BaseEdge,
  EdgeLabelRenderer,
  EdgeProps,
  getBezierPath,
} from "@xyflow/react";

import { useEditorContext } from "@flow/features/Editor/editorContext";
import { Edge } from "@flow/types";

import useHooks from "./hooks";

export type CustomEdgeProps = EdgeProps<Edge>;

const DefaultEdge: React.FC<CustomEdgeProps> = ({
  id,
  source,
  target,
  sourceX,
  sourceY,
  sourcePosition,
  sourceHandleId,
  targetX,
  targetY,
  targetPosition,
  selected,
  // markerEnd,
  // ...props
}) => {
  const editorContext = useEditorContext();
  const readonly = editorContext?.readonly ?? false;

  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const {
    // sourceNodeStatus,
    jobStatus,
    intermediateDataIsSet,
    hasIntermediateData,
    handleDoubleClick,
  } = useHooks({
    id,
    source,
    sourceHandleId,
    target,
    selected,
  });

  return (
    <>
      <BaseEdge id={id} path={edgePath} />
      {!readonly && (
        <>
          <EdgeLabelRenderer>
            {jobStatus === "failed" && (
              <XIcon
                className="nodrag nopan absolute size-[20px] origin-center rounded-full border border-destructive bg-primary fill-destructive p-1"
                weight="bold"
                style={{
                  pointerEvents: "all",
                  transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
                }}
              />
            )}
            {hasIntermediateData && (
              <TableIcon
                className={`nodrag nopan absolute size-[25px] origin-center rounded-full border bg-primary p-1 transition-[height,width] hover:size-[40px] hover:fill-success  ${intermediateDataIsSet ? "size-[35px] border-success bg-success fill-white hover:fill-white" : selected ? "border-success fill-success" : "border-slate-400/80 fill-success/80"}`}
                style={{
                  pointerEvents: "all",
                  transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
                }}
                onDoubleClick={handleDoubleClick}
              />
            )}
          </EdgeLabelRenderer>
          {jobStatus === "completed" && (
            <path
              className="stroke-success"
              d={edgePath}
              strokeWidth="1"
              fill="none"
              markerEnd="url(#arrow)"
            />
          )}
          {jobStatus === "queued" && (
            <path d={edgePath} stroke="#27272A" fill="none" className="pulse" />
          )}
          {jobStatus === "running" && (
            <>
              <path
                d={edgePath}
                stroke="#27272A"
                strokeDasharray="10,10"
                fill="none">
                <animate
                  attributeName="stroke-dashoffset"
                  from="40"
                  to="0"
                  dur="3s"
                  repeatCount="indefinite"
                />
              </path>
              <g>
                <circle className="opacity-25" r="6" fill="#ffffff">
                  <animateMotion
                    dur="6s"
                    repeatCount="indefinite"
                    path={edgePath}
                  />
                </circle>
                <circle
                  className="opacity-75"
                  style={{ filter: `drop-shadow(3px 3px 5px #471a27)` }}
                  r="3"
                  fill="#bbffff">
                  <animateMotion
                    dur="6s"
                    repeatCount="indefinite"
                    path={edgePath}
                  />
                </circle>
              </g>
            </>
          )}
          {/* {sourceNodeStatus === "failed" && (
            <path d={edgePath} stroke="#fc4444" strokeWidth="1" fill="none" />
          )} */}
        </>
      )}
    </>
  );
};

export default DefaultEdge;
