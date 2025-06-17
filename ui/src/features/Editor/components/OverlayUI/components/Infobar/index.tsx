import { memo, useEffect, useState } from "react";

import { useT } from "@flow/lib/i18n";
import { Edge, Node } from "@flow/types";

type Props = {
  hoveredDetails: Node | Edge;
};

const Infobar: React.FC<Props> = ({ hoveredDetails }) => {
  const t = useT();
  const [isHovered, setIsHovered] = useState(false);

  useEffect(() => {
    const timeout = setTimeout(() => {
      setIsHovered(true);
    }, 500);

    return () => {
      clearTimeout(timeout);
      setIsHovered(false);
    };
  }, []);
  return isHovered ? (
    <div className="absolute bottom-4 left-1/2 z-10 -translate-x-1/2 rounded-md bg-secondary/80 p-1 shadow-md backdrop-blur-sm">
      <div className="flex justify-center gap-5 rounded-md px-4 py-2">
        {"source" in hoveredDetails ? (
          <div className="flex flex-col items-center gap-1">
            <p className="text-xs font-bold">{t("Edge ID: ")}</p>
            <p className="text-xs font-light">{hoveredDetails.id}</p>
            <div className="flex items-center gap-2">
              <div className="flex flex-col items-center gap-1">
                <p className="text-xs font-bold">{t("Source Node ID: ")}</p>
                <p className="text-xs font-light">{hoveredDetails.source}</p>
              </div>
              <p className="text-xs font-bold">{" -> "}</p>
              <div className="flex flex-col items-center gap-1">
                <p className="text-xs font-bold">{t("Target Node ID: ")}</p>
                <p className="text-xs font-light">{hoveredDetails.target}</p>
              </div>
            </div>
          </div>
        ) : (
          <div className="flex flex-col items-center gap-1">
            <p className="text-xs font-bold">{t("Node ID: ")}</p>
            <p className="text-xs font-light">{hoveredDetails.id}</p>
            <div className="flex w-full justify-between gap-1">
              <div className="flex gap-1">
                <p className="text-xs font-bold">{t("Name: ")}</p>
                <p className="text-xs font-light">
                  {hoveredDetails.data.customizations?.customName ||
                    hoveredDetails.data.officialName}
                </p>
              </div>
              <div className="flex gap-1">
                <p className="text-xs font-bold">{t("Type: ")}</p>
                <p className="text-xs font-light">{hoveredDetails.type}</p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  ) : null;
};

export default memo(Infobar);
