import { memo, useRef, useState } from "react";

import {
  Button,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Slider,
} from "@flow/components";
import {
  DEFAULT_GRID_SIZE,
  DEFAULT_LAYOUT_X_SPACING,
  DEFAULT_LAYOUT_Y_SPACING,
} from "@flow/global-constants";
import { useT } from "@flow/lib/i18n";
import type { Algorithm, Direction } from "@flow/types";

type Props = {
  onLayoutChange: (
    algorithm: Algorithm,
    direction: Direction,
    xSpacing: number,
    ySpacing: number,
  ) => void;
  onSpacingChange: (xScale: number, yScale: number) => void;
};

const LayoutSubToolbar: React.FC<Props> = ({
  onLayoutChange,
  onSpacingChange,
}) => {
  const t = useT();
  const [direction, setDirection] = useState<Direction>("Horizontal");
  const [xSpacing, setXSpacing] = useState(DEFAULT_LAYOUT_X_SPACING);
  const [ySpacing, setYSpacing] = useState(DEFAULT_LAYOUT_Y_SPACING);

  const committedXRef = useRef(DEFAULT_LAYOUT_X_SPACING);
  const committedYRef = useRef(DEFAULT_LAYOUT_Y_SPACING);

  const min = DEFAULT_GRID_SIZE;
  const max = DEFAULT_GRID_SIZE * 20;
  const step = DEFAULT_GRID_SIZE;

  const handleCleanUp = () => {
    onLayoutChange("dagre", direction, xSpacing, ySpacing);
    committedXRef.current = xSpacing;
    committedYRef.current = ySpacing;
  };

  return (
    <div className="flex items-center gap-4 rounded-md border border-accent bg-primary/50 px-4 py-2 text-popover-foreground shadow-md backdrop-blur">
      <Button variant="outline" onClick={handleCleanUp}>
        {t("Clean up")}
      </Button>
      <div className="h-5 border-r border-border" />
      <Select
        value={direction}
        onValueChange={(v) => setDirection(v as Direction)}>
        <SelectTrigger className="h-7 w-30 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="Horizontal">{t("Horizontal")}</SelectItem>
          <SelectItem value="Vertical">{t("Vertical")}</SelectItem>
        </SelectContent>
      </Select>
      <div className="h-5 border-r border-border" />
      <div className="flex min-w-45 flex-col gap-1.5">
        <div className="flex items-center justify-between">
          <Label className="text-xs">{t("X Spacing")}</Label>
          <span className="text-xs text-muted-foreground">
            {Math.round(xSpacing)}
          </span>
        </div>
        <Slider
          min={min}
          max={max}
          step={step}
          value={[xSpacing]}
          onValueChange={([v]) => setXSpacing(v)}
          onValueCommit={([v]) => {
            const scale = v / committedXRef.current;
            committedXRef.current = v;
            onSpacingChange(scale, 1);
          }}
        />
      </div>
      <div className="flex min-w-45 flex-col gap-1.5">
        <div className="flex items-center justify-between">
          <Label className="text-xs">{t("Y Spacing")}</Label>
          <span className="text-xs text-muted-foreground">
            {Math.round(ySpacing)}
          </span>
        </div>
        <Slider
          min={min}
          max={max}
          step={step}
          value={[ySpacing]}
          onValueChange={([v]) => setYSpacing(v)}
          onValueCommit={([v]) => {
            const scale = v / committedYRef.current;
            committedYRef.current = v;
            onSpacingChange(1, scale);
          }}
        />
      </div>
    </div>
  );
};

export default memo(LayoutSubToolbar);
