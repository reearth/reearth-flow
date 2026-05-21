import { memo, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogTitle,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Algorithm, Direction } from "@flow/types";

type Props = {
  onClose: () => void;
  onLayoutChange: (
    algorithm: Algorithm,
    direction: Direction,
    spacing: number,
  ) => void;
};

const LayoutOptionsDialog: React.FC<Props> = ({ onClose, onLayoutChange }) => {
  const t = useT();

  const [algorithm, setAlgorithm] = useState<Algorithm>("dagre");
  const [layoutDirection, setLayoutDirection] =
    useState<Direction>("Horizontal");
  const [spacing] = useState(100);

  const algorithms: Record<Algorithm, string> = {
    dagre: t("Dagre (Tree)"),
    elk: t("ELK (Layered)"),
    d3: t("D3 Hierarchy"),
  };

  const layoutDirections: Record<Direction, string> = {
    Horizontal: t("Horizontal"),
    Vertical: t("Vertical"),
  };

  const handleLayoutChange = () => {
    onLayoutChange(algorithm, layoutDirection, spacing);
    onClose();
  };

  return (
    <Dialog open={true} onOpenChange={(o) => !o && onClose()}>
      <DialogContent size="sm" position="top" overlayBgClass="bg-opacity-0">
        <DialogTitle>{t("Layout Options")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex-row">
            <div className="flex-1">
              <Label>{t("Algorithm: ")}</Label>
              <div className="ml-2">
                <Select
                  value={algorithm}
                  onValueChange={(v) => setAlgorithm(v as Algorithm)}>
                  <SelectTrigger className="h-8 w-37.5">
                    <SelectValue placeholder={algorithms.dagre} />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(algorithms).map(([value, label]) => (
                      <SelectItem key={value} value={value}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </DialogContentSection>
          <DialogContentSection className="flex-row">
            <div className="flex-1">
              <Label>{t("Direction: ")}</Label>
              <div className="ml-2">
                <Select
                  value={layoutDirection}
                  onValueChange={(v) => setLayoutDirection(v as Direction)}>
                  <SelectTrigger className="h-8 w-37.5">
                    <SelectValue placeholder={layoutDirections.Horizontal} />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(layoutDirections).map(([value, label]) => (
                      <SelectItem key={value} value={value}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button onClick={handleLayoutChange}>{t("Update")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default memo(LayoutOptionsDialog);
