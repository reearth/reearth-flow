import { SlidersHorizontalIcon } from "@phosphor-icons/react";
import { useState } from "react";

import { formatVariableValue } from "@flow/features/common/formatVariableValue";
import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";

import CopyButton from "./CopyButton";
import type { DebugRunVariable } from "./debugRunVariables";

const toText = (value: unknown, empty: string) => {
  const formatted = formatVariableValue(value, empty);
  return Array.isArray(formatted) ? formatted.join(", ") : formatted;
};

// The workflow variables the run used. The summary around it scrolls, so a
// long list never runs into the debug panel.
const VariablesSection: React.FC<{ variables: DebugRunVariable[] }> = ({
  variables,
}) => {
  const t = useT();
  const changedCount = variables.filter((v) => v.changedFrom).length;

  return (
    <div className="border-t border-border/50 pt-1 dark:border-primary/50">
      <div className="flex h-7 items-center gap-2 px-2">
        <SlidersHorizontalIcon weight="light" size={14} className="shrink-0" />
        <span className="truncate text-xs dark:font-light">
          {t("Workflow Variables")}
        </span>
        <span className="rounded-full bg-accent/50 px-1.5 text-[10px] tabular-nums">
          {variables.length}
        </span>
        <span className="flex-1" />
        {changedCount > 0 && (
          <span className="shrink-0 text-[10px]">
            {t("{{count}} changed", { count: changedCount })}
          </span>
        )}
      </div>
      <ul className="flex flex-col">
        {variables.map((variable) => (
          <VariableRow key={variable.key} variable={variable} />
        ))}
      </ul>
    </div>
  );
};

// One variable: its name, then its value on its own line so a long value gets
// the full width. Clicking the row shows the whole value (and the default it
// replaced); the copy button takes the whole value either way.
const VariableRow: React.FC<{ variable: DebugRunVariable }> = ({
  variable: { name, value, changedFrom },
}) => {
  const t = useT();
  const [open, setOpen] = useState(false);
  const text = toText(value, t("Not set"));

  return (
    <li className="group relative rounded-lg hover:bg-accent/30">
      <button
        type="button"
        className="flex w-full cursor-pointer flex-col gap-0.5 px-2 py-1 pr-8 text-left"
        aria-expanded={open}
        onClick={() => setOpen((o) => !o)}>
        <span className="flex w-full items-center gap-1.5 text-[10px] opacity-80">
          <span className="truncate">{name}</span>
          {changedFrom && (
            <span className="shrink-0 rounded-full bg-accent/50 px-1.5">
              {t("Changed")}
            </span>
          )}
        </span>
        <span
          className={cn(
            "w-full text-xs font-light",
            changedFrom && "text-foreground",
            open ? "break-all whitespace-pre-wrap" : "truncate",
          )}>
          {text}
        </span>
        {open && changedFrom && (
          <span className="w-full text-[10px] break-all opacity-80">
            {t("Default: {{value}}", {
              value: toText(changedFrom.defaultValue, t("Not set")),
            })}
          </span>
        )}
      </button>
      <CopyButton
        text={text}
        className="absolute top-1 right-1 size-6 opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
      />
    </li>
  );
};

export default VariablesSection;
