import { memo, useState } from "react";

import {
  Button,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Algorithm, Direction } from "@flow/types";

type Props = {
  onLayoutChange: (algorithm: Algorithm, direction: Direction) => void;
};

const LayoutSubToolbar: React.FC<Props> = ({ onLayoutChange }) => {
  const t = useT();
  const [direction, setDirection] = useState<Direction>("Horizontal");

  const handleCleanUp = () => {
    onLayoutChange("dagre", direction);
  };

  return (
    <div className="flex items-center gap-4 rounded-md border border-accent bg-primary/50 px-4 py-2 text-popover-foreground shadow-md backdrop-blur">
      <Button variant="outline" onClick={handleCleanUp}>
        {t("Clean up")}
      </Button>
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
    </div>
  );
};

export default memo(LayoutSubToolbar);
