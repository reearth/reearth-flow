import {
  useNodes,
  useReactFlow,
  ViewportPortal,
  XYPosition,
} from "@xyflow/react";

export default function NodeInspector() {
  const { getInternalNode } = useReactFlow();
  const nodes = useNodes();

  return (
    <ViewportPortal>
      <div className="text-secondary-foreground">
        {nodes
          .filter((node) => node.selected)
          .map((node) => {
            const internalNode = getInternalNode(node.id);
            if (!internalNode) {
              return null;
            }

            const absPosition = internalNode?.internals.positionAbsolute;
            return (
              <NodeInfo
                key={node.id}
                id={node.id}
                parentId={node?.parentId}
                selected={!!node.selected}
                type={node.type || "default"}
                position={node.position}
                absPosition={absPosition}
                width={node.measured?.width ?? 0}
                height={node.measured?.height ?? 0}
                data={node.data}
              />
            );
          })}
      </div>
    </ViewportPortal>
  );
}

type NodeInfoProps = {
  id: string;
  parentId?: string;
  type: string;
  selected: boolean;
  position: XYPosition;
  absPosition: XYPosition;
  width?: number;
  height?: number;
  data: any;
};

const NodeInfo = ({
  id,
  parentId,
  type,
  selected,
  position,
  absPosition,
  width,
  height,
  data,
}: NodeInfoProps) => {
  if (!width || !height) return null;

  const absoluteTransform = `translate(${absPosition.x}px, ${absPosition.y + height}px)`;
  const formattedPosition = `${position.x.toFixed(1)}, ${position.y.toFixed(1)}`;
  const formattedDimensions = `${width} × ${height}`;
  const selectionStatus = selected ? "Selected" : "Not Selected";

  return (
    <div
      className="absolute z-1000 max-w-[320px] min-w-[180px] rounded-lg border border-slate-700 bg-slate-800/95 p-4 text-xs text-white shadow-lg"
      style={{
        transform: absoluteTransform,
      }}>
      <div className="mb-2 text-sm font-semibold">Node Inspector</div>
      <div className="mb-1">
        <span className="font-medium">ID:</span> {id}
      </div>
      <div className="mb-1">
        <span className="font-medium">Type:</span> {type}
      </div>
      {parentId && (
        <div className="mb-1">
          <span className="font-medium">Parent ID:</span> {parentId}
        </div>
      )}
      <div className="mb-1">
        <span className="font-medium">Status:</span>{" "}
        <span
          className={
            selected
              ? "font-semibold text-green-500"
              : "font-semibold text-red-400"
          }>
          {selectionStatus}
        </span>
      </div>
      <div className="mb-1">
        <span className="font-medium">Position:</span> {formattedPosition}
      </div>
      <div className="mb-1">
        <span className="font-medium">Dimensions:</span> {formattedDimensions}
      </div>
      <div>
        <span className="font-medium">Data:</span>
        <pre className="min-w-0 overflow-x-auto rounded bg-slate-700/70 px-2 py-1 text-[11px]">
          {JSON.stringify(data, null, 2)}
        </pre>
      </div>
    </div>
  );
};
