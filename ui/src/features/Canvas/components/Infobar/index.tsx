import type { Edge, Node } from "reactflow";

type Props = {
  className?: string;
  hoveredDetails: Node | Edge | undefined;
};

const Infobar: React.FC<Props> = ({ className, hoveredDetails }) => {
  return hoveredDetails ? (
    <div className={`flex justify-center gap-5 bg-zinc-800 rounded-md py-2 px-4 z-10 ${className}`}>
      {"source" in hoveredDetails ? (
        <>
          <p className="text-xs text-zinc-400">Source ID: {hoveredDetails.source}</p>
          <p className="text-xs text-zinc-400">{" -> "}</p>
          <p className="text-xs text-zinc-400">Target ID: {hoveredDetails.target}</p>
        </>
      ) : (
        <>
          <p className="text-xs text-zinc-400">ID: {hoveredDetails.id}</p>
          <p className="text-xs text-zinc-400">Name: {hoveredDetails.data?.name}</p>
          <p className="text-xs text-zinc-400">Type: {hoveredDetails.type}</p>
        </>
      )}
    </div>
  ) : null;
};

export { Infobar };
