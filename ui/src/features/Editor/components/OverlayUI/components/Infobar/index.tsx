import { memo, useEffect, useState } from "react";

import { Edge, Node } from "@flow/types";

type Props = {
  hoveredDetails: Node | Edge;
};

const Infobar: React.FC<Props> = ({ hoveredDetails }) => {
  const [isHovered, setIsHovered] = useState(false);

  useEffect(() => {
    const timeout = setTimeout(() => {
      setIsHovered(true);
    }, 1000);

    return () => {
      clearTimeout(timeout);
      setIsHovered(false);
    };
  }, []);
  return isHovered ? (
    <div className="absolute bottom-1 left-1/2 z-10 -translate-x-1/2 rounded-md border bg-primary">
      <div className="flex justify-center gap-5 rounded-md px-4 py-2">
        {"source" in hoveredDetails ? (
          <>
            <p className="text-xs">Source ID: {hoveredDetails.source}</p>
            <p className="text-xs">{" -> "}</p>
            <p className="text-xs">Target ID: {hoveredDetails.target}</p>
          </>
        ) : (
          <>
            <p className="text-xs">ID: {hoveredDetails.id}</p>
            <p className="text-xs">
              Name:{" "}
              {hoveredDetails.data.customizations?.customName ||
                hoveredDetails.data.officialName}
            </p>
            <p className="text-xs">Type: {hoveredDetails.type}</p>
          </>
        )}
      </div>
    </div>
  ) : null;
};

export default memo(Infobar);
