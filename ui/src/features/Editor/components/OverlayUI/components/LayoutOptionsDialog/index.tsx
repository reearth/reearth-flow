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
  isOpen: boolean;
  onClose: () => void;
  onLayoutChange: (
    algorithm: Algorithm,
    direction: Direction,
    spacing: number,
  ) => void;
};

const LayoutOptionsDialog: React.FC<Props> = ({
  isOpen,
  onClose,
  onLayoutChange,
}) => {
  const t = useT();

  const [layoutDirection, setLayoutDirection] =
    useState<Direction>("Horizontal");
  const [spacing, _setSpacing] = useState(100);

  const layoutDirections = {
    Horizontal: t("Horizontal"),
    Vertical: t("Vertical"),
  };

  const handleDirectionChange = (value: Direction) => {
    setLayoutDirection(value);
  };

  // const handleSpacingChange = (e: ChangeEvent<HTMLInputElement>) => {
  //   if (Number.isNaN(e.target.value)) return;
  //   if (e.target.valueAsNumber > 500) {
  //     setSpacing(500);
  //   } else {
  //     setSpacing(Number(e.target.value));
  //   }
  // };

  const handleLayoutChange = () => {
    onLayoutChange("dagre", layoutDirection, spacing);
    onClose();
  };

  return (
    <Dialog open={isOpen} onOpenChange={(o) => !o && onClose()}>
      <DialogContent size="sm" position="top" overlayBgClass="bg-opacity-0">
        <DialogTitle>{t("Layout Options")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection>
            <Label>{t("Algorithm: ")}</Label>
            <div className="ml-2">
              <p>Dagre tree</p>
            </div>
          </DialogContentSection>
          <DialogContentSection className="flex-row">
            <div className="flex-1">
              <Label>{t("Direction: ")}</Label>
              <div className="ml-2">
                <Select
                  value={layoutDirection}
                  onValueChange={handleDirectionChange}>
                  <SelectTrigger className="h-[32px] w-[150px]">
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
            {/* <div className="flex-1">
              <Label>{t("Spacing: ")}</Label>
              <div className="ml-2 w-[100px]">
                <div className="flex items-center gap-1">
                  <Input
                    type="number"
                    max={500}
                    value={spacing}
                    onChange={handleSpacingChange}
                  />
                  <p>px</p>
                </div>
              </div>
            </div> */}
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
