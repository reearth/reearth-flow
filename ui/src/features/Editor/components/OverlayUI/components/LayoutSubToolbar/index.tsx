import { memo, useEffect, useMemo, useState } from "react";
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
import type { Algorithm, Direction, YDocMetadataValue } from "@flow/types";

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
  const yMetadata = useMemo(
    () => Ydoc?.getMap<YDocMetadataValue>("metadata"),
    [Ydoc],
  );

  const metadata = useY(yMetadata ?? new YMap<YDocMetadataValue>()) as Record<
    string,
    any
  >;

  const [direction, setDirection] = useState<Direction>(
    () => (metadata?.layoutDirection as Direction) || "Horizontal",
  );
  const [applyToAll, setApplyToAll] = useState<boolean>(
    () => metadata?.layoutApplyToAll ?? false,
  );

  const handleDirectionChange = (newDirection: Direction) => {
    yMetadata?.set("layoutDirection", newDirection);
    setDirection(newDirection);
  };

  const handleApplyToAllChange = (newValue: boolean) => {
    yMetadata?.set("layoutApplyToAll", newValue);
    setApplyToAll(newValue);
  };

  const handleCleanUp = () => {
    onLayoutChange("dagre", direction, applyToAll);
    onClose();
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [onClose]);

  return (
    <div className="flex items-center gap-4 rounded-xl border border-primary bg-primary/50 p-1 text-popover-foreground shadow-md shadow-[black]/10 backdrop-blur dark:shadow-secondary">
      <Select value={direction} onValueChange={handleDirectionChange}>
        <SelectTrigger className="h-7 w-30 text-xs">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="Horizontal">
            <p className="select-none">{t("Horizontal")}</p>
          </SelectItem>
          <SelectItem value="Vertical">
            <p className="select-none">{t("Vertical")}</p>
          </SelectItem>
        </SelectContent>
      </Select>
      <label className="flex cursor-pointer items-center gap-2 text-xs select-none">
        <Checkbox
          className="bg-accent"
          checked={applyToAll}
          onCheckedChange={handleApplyToAllChange}
        />
        {t("Apply to all workflows")}
      </label>
      <div className="h-5 border-r border-border" />
      <Button className="h-8 max-w-fit select-none" onClick={handleCleanUp}>
        {t("Clean up")}
      </Button>
    </div>
  );
};

export default memo(LayoutSubToolbar);
