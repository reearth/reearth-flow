import { CopyrightIcon } from "@phosphor-icons/react";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

const AttributionsDialog: React.FC<Props> = ({ isOpen, onOpenChange }) => {
  const t = useT();
  const attributions = [
    {
      name: "ReactFlow (@xyflow/react) - MIT License",
      description: "Node-based workflow visualization",
      url: "https://reactflow.dev",
    },
    {
      name: "Cesium (cesium) - Apache License 2.0",
      description: "3D geospatial visualization engine",
      url: "https://cesium.com",
    },
    {
      name: "MapLibre GL JS (maplibre-gl) - BSD-3-Clause License",
      description: "Interactive vector maps in web browsers",
      url: "https://maplibre.org",
    },
  ];

  return (
    <Dialog open={isOpen} onOpenChange={(o) => onOpenChange(o)}>
      <DialogContent className="max-h-[800px] w-full max-w-4xl overflow-hidden">
        <DialogTitle className="flex items-center gap-2">
          <CopyrightIcon /> {t("Attributions")}
        </DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection>
            {attributions.map((attr, idx) => (
              <div key={attr.name} className="mb-4">
                <h3 className="font-semibold">{attr.name}</h3>
                <p className="italic">{attr.description}</p>
                <a
                  href={attr.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-muted-foreground hover:underline">
                  {attr.url}
                </a>
              </div>
            ))}
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
    </Dialog>
  );
};

export { AttributionsDialog };
