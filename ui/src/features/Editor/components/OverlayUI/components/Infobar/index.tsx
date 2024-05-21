import type { Edge, Node } from "reactflow";

type Props = {
  hoveredDetails: Node | Edge | undefined;
};

const Infobar: React.FC<Props> = ({ hoveredDetails }) => {
  return hoveredDetails ? (
    <div className="flex justify-center gap-5 bg-zinc-800 rounded-md py-2 px-4 z-10 absolute bottom-2 left-[50%] translate-x-[-50%]">
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
