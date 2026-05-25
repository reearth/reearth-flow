import { memo, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogTitle,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Algorithm, Direction } from "@flow/types";
import { DEFAULT_LAYOUT_SPACING } from "@flow/utils/autoLayout";

type Props = {
  onClose: () => void;
  onLayoutChange: (
    algorithm: Algorithm,
    direction: Direction,
    xSpacing: number,
    ySpacing: number,
  ) => void;
};

const LayoutOptionsDialog: React.FC<Props> = ({ onClose, onLayoutChange }) => {
  const t = useT();

  const [algorithm, setAlgorithm] = useState<Algorithm>("dagre");
  const [layoutDirection, setLayoutDirection] =
    useState<Direction>("Horizontal");
  const [xSpacing, setXSpacing] = useState(DEFAULT_LAYOUT_SPACING.x);
  const [ySpacing, setYSpacing] = useState(DEFAULT_LAYOUT_SPACING.y);

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
    onLayoutChange(algorithm, layoutDirection, xSpacing, ySpacing);
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
          <DialogContentSection className="flex-row gap-4">
            <div className="flex flex-1 items-center gap-2">
              <Label className="shrink-0">{t("X Spacing: ")}</Label>
              <Input
                type="number"
                min={0}
                max={500}
                className="h-8 w-20"
                value={xSpacing}
                onChange={(e) =>
                  setXSpacing(Math.max(0, Number(e.target.value)))
                }
              />
            </div>
            <div className="flex flex-1 items-center gap-2">
              <Label className="shrink-0">{t("Y Spacing: ")}</Label>
              <Input
                type="number"
                min={0}
                max={500}
                className="h-8 w-20"
                value={ySpacing}
                onChange={(e) =>
                  setYSpacing(Math.max(0, Number(e.target.value)))
                }
              />
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
