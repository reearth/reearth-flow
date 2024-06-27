import { Edge, Node } from "@flow/types";

type Props = {
  hoveredDetails: Node | Edge | undefined;
};

const Infobar: React.FC<Props> = ({ hoveredDetails }) => {
  return hoveredDetails ? (
    <div className="bg-zinc-800 absolute bottom-1 left-[50%] translate-x-[-50%] z-10 rounded-md border border-zinc-700">
      <div className="flex justify-center gap-5 bg-zinc-900/50 rounded-md py-2 px-4">
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
    </div>
  ) : null;
};

export { Infobar };
