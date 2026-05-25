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
import type { Direction } from "@flow/types";
import { DEFAULT_LAYOUT_SPACING } from "@flow/utils/autoLayout";

type Props = {
  onClose: () => void;
  onLayoutChange: (
    direction: Direction,
    xSpacing: number,
    ySpacing: number,
  ) => Promise<void>;
};

const LayoutOptionsDialog: React.FC<Props> = ({ onClose, onLayoutChange }) => {
  const t = useT();

  const [layoutDirection, setLayoutDirection] =
    useState<Direction>("Horizontal");
  const [xSpacing, setXSpacing] = useState(DEFAULT_LAYOUT_SPACING.x);
  const [ySpacing, setYSpacing] = useState(DEFAULT_LAYOUT_SPACING.y);

  const layoutDirections: Record<Direction, string> = {
    Horizontal: t("Horizontal"),
    Vertical: t("Vertical"),
  };

  const handleLayoutChange = async () => {
    await onLayoutChange(layoutDirection, xSpacing, ySpacing);
    onClose();
  };

  return (
    <Dialog open={true} onOpenChange={(o) => !o && onClose()}>
      <DialogContent size="sm" position="top" overlayBgClass="bg-opacity-0">
        <DialogTitle>{t("Layout Options")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex-row">
            <div className="flex-1">
              <Label>{t("Direction: ")}</Label>
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
                onChange={(e) => {
                  const value = parseFloat(e.target.value);
                  setXSpacing(Math.max(0, value));
                }}
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
                onChange={(e) => {
                  const value = parseFloat(e.target.value);
                  setYSpacing(Math.max(0, value));
                }}
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
