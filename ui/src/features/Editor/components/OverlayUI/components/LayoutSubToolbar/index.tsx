import { memo, useMemo, useState } from "react";
import { useY } from "react-yjs";
import { Doc, Map as YMap } from "yjs";

import {
  Button,
  Checkbox,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Algorithm, Direction } from "@flow/types";

type Props = {
  Ydoc: Doc | null;
  onLayoutChange: (
    algorithm: Algorithm,
    direction: Direction,
    applyToAll: boolean,
  ) => void;
  onClose: () => void;
};

const LayoutSubToolbar: React.FC<Props> = ({
  Ydoc,
  onLayoutChange,
  onClose,
}) => {
  const t = useT();
  const yMetadata = useMemo(() => Ydoc?.getMap<any>("metadata"), [Ydoc]);
  const metadata = useY(yMetadata ?? new YMap()) as Record<string, any>;

  const [direction, setDirection] = useState<Direction>(
    () => (metadata?.layoutDirection as Direction) || "Horizontal",
  );
  const [applyToAll, setApplyToAll] = useState<boolean>(
    () => metadata?.layoutApplyToAll ?? false,
  );

  const handleCleanUp = () => {
    yMetadata?.set("layoutDirection", direction);
    yMetadata?.set("layoutApplyToAll", applyToAll);
    onLayoutChange("dagre", direction, applyToAll);
    onClose();
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
      <label className="flex cursor-pointer items-center gap-2 text-xs text-accent-foreground select-none">
        <Checkbox
          className="bg-accent"
          checked={applyToAll}
          onCheckedChange={(v) => setApplyToAll(v === true)}
        />
        {t("Include subworkflows")}
      </label>
    </div>
  );
};

export default memo(LayoutSubToolbar);
