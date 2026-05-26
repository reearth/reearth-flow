import { memo, useRef, useState } from "react";

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
  Slider,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Direction, Workflow } from "@flow/types";
import { DEFAULT_LAYOUT_SPACING } from "@flow/utils/autoLayout";

type Props = {
  rawWorkflows: Workflow[];
  onClose: () => void;
  onLayoutChange: (
    direction: Direction,
    xSpacing: number,
    ySpacing: number,
  ) => Promise<void>;
  onLayoutPreview: (
    direction: Direction,
    xSpacing: number,
    ySpacing: number,
  ) => Promise<void>;
  onCancelLayoutPreview: (originalWorkflows: Workflow[]) => void;
};

const LayoutOptionsDialog: React.FC<Props> = ({
  rawWorkflows,
  onClose,
  onLayoutChange,
  onLayoutPreview,
  onCancelLayoutPreview,
}) => {
  const t = useT();

  // Snapshot taken once on mount, before any preview changes touch Yjs.
  const snapshotRef = useRef<Workflow[]>(rawWorkflows);

  const [layoutDirection, setLayoutDirection] =
    useState<Direction>("Horizontal");
  const [xSpacing, setXSpacing] = useState(DEFAULT_LAYOUT_SPACING.x);
  const [ySpacing, setYSpacing] = useState(DEFAULT_LAYOUT_SPACING.y);

  const layoutDirections: Record<Direction, string> = {
    Horizontal: t("Horizontal"),
    Vertical: t("Vertical"),
  };

  const handleDirectionChange = (v: Direction) => {
    setLayoutDirection(v);
    onLayoutPreview(v, xSpacing, ySpacing);
  };

  const handleXChange = (value: number[]) => {
    const next = value[0];
    setXSpacing(next);
    onLayoutPreview(layoutDirection, next, ySpacing);
  };

  const handleYChange = (value: number[]) => {
    const next = value[0];
    setYSpacing(next);
    onLayoutPreview(layoutDirection, xSpacing, next);
  };

  const handleCancel = () => {
    onCancelLayoutPreview(snapshotRef.current);
    onClose();
  };

  const handleSubmit = async () => {
    await onLayoutChange(layoutDirection, xSpacing, ySpacing);
    onClose();
  };

  return (
    <Dialog open={true} onOpenChange={(o) => !o && handleCancel()}>
      <DialogContent size="sm" position="top" overlayBgClass="bg-opacity-0">
        <DialogTitle>{t("Layout Options")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex-row">
            <div className="flex-1">
              <Label>{t("Direction: ")}</Label>
              <Select
                value={layoutDirection}
                onValueChange={(v) => handleDirectionChange(v as Direction)}>
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
          </DialogContentSection>
          <DialogContentSection className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <div className="flex items-center justify-between">
                <Label>{t("X Spacing")}</Label>
                <span className="w-8 text-right text-xs text-muted-foreground">
                  {xSpacing}
                </span>
              </div>
              <Slider
                min={0}
                max={500}
                step={1}
                value={[xSpacing]}
                onValueChange={handleXChange}
              />
            </div>
            <div className="flex flex-col gap-2">
              <div className="flex items-center justify-between">
                <Label>{t("Y Spacing")}</Label>
                <span className="w-8 text-right text-xs text-muted-foreground">
                  {ySpacing}
                </span>
              </div>
              <Slider
                min={0}
                max={500}
                step={1}
                value={[ySpacing]}
                onValueChange={handleYChange}
              />
            </div>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button variant="outline" onClick={handleCancel}>
            {t("Cancel")}
          </Button>
          <Button onClick={handleSubmit}>{t("Apply")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default memo(LayoutOptionsDialog);
