import { CaretUpIcon, ListBulletsIcon } from "@phosphor-icons/react";
import { memo, useState } from "react";

import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";

export type LegendEntry = { color: string; label: string };

type Props = {
  entries: LegendEntry[];
  className?: string;
};

const Swatch: React.FC<{ color: string }> = ({ color }) => (
  <span
    className="size-2.5 shrink-0 rounded-sm"
    style={{ backgroundColor: color }}
  />
);

/**
 * What a view's colours mean. A single entry is shown as it is; more than one
 * fold away behind a button showing their colours, so they cover the view
 * only when asked for.
 */
const Legend: React.FC<Props> = ({ entries, className }) => {
  const t = useT();
  const [open, setOpen] = useState(false);
  const box = "rounded bg-card/90 px-2 py-1 text-xs text-muted-foreground";

  if (entries.length === 0) return null;

  if (entries.length === 1) {
    const [{ color, label }] = entries;
    return (
      <div
        className={cn(
          "pointer-events-none flex items-center gap-1.5",
          box,
          className,
        )}>
        <Swatch color={color} />
        {label}
      </div>
    );
  }

  if (!open) {
    return (
      <button
        type="button"
        className={cn(
          "flex items-center gap-1.5 hover:text-foreground",
          box,
          className,
        )}
        aria-expanded={false}
        aria-label={t("Show legend")}
        onClick={() => setOpen(true)}>
        <ListBulletsIcon size={12} />
        {entries.map(({ color, label }) => (
          <Swatch key={label} color={color} />
        ))}
      </button>
    );
  }

  return (
    <div className={cn("flex flex-col gap-1", box, className)}>
      <button
        type="button"
        className="flex items-center justify-between gap-2 hover:text-foreground"
        aria-expanded
        aria-label={t("Hide legend")}
        onClick={() => setOpen(false)}>
        <span className="font-medium">{t("Legend")}</span>
        <CaretUpIcon size={12} />
      </button>
      <ul className="flex flex-col gap-1">
        {entries.map(({ color, label }) => (
          <li key={label} className="flex items-center gap-1.5">
            <Swatch color={color} />
            {label}
          </li>
        ))}
      </ul>
    </div>
  );
};

export default memo(Legend);
