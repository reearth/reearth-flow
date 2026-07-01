import {
  CircleNotchIcon,
  InfoIcon,
  WarningCircleIcon,
} from "@phosphor-icons/react";
import { memo, useMemo } from "react";

import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@flow/components";
import { TYPE_COLOR } from "@flow/features/Editor/components/ParamsDialog/components/ValueEditorDialog/components/flowExprConstants";
import { useT } from "@flow/lib/i18n";
import type { FieldReport, NodeSchemaMeta } from "@flow/types";

type Props = {
  schema?: NodeSchemaMeta;
};

const collectFields = (schema?: NodeSchemaMeta): FieldReport[] => {
  if (!schema) return [];
  const seen = new Set<string>();
  const fields: FieldReport[] = [];
  Object.values(schema.ports ?? {}).forEach((port) => {
    port.fields.forEach((field) => {
      if (seen.has(field.name)) return;
      seen.add(field.name);
      fields.push(field);
    });
  });
  return fields;
};

const SchemaIndicator: React.FC<Props> = ({ schema }) => {
  const t = useT();

  const fields = useMemo(() => collectFields(schema), [schema]);
  if (schema?.status === "running") {
    return (
      <div className="flex translate-x-0.5 items-center">
        <CircleNotchIcon className="size-3 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (schema?.status === "failed") {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="flex translate-x-0.5 items-center">
              <WarningCircleIcon className="size-3 text-warning" />
            </div>
          </TooltipTrigger>
          <TooltipContent side="bottom" className="max-w-64">
            <div className="flex flex-col gap-1">
              <span>{t("Schema preview failed. Re-save to retry.")}</span>
              {schema.note && (
                <span className="break-wordstext-muted-foreground font-mono text-[10px]">
                  {schema.note}
                </span>
              )}
            </div>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  if (!schema) return null;

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <div className="flex translate-x-0.5 items-center">
            <InfoIcon className="size-3 text-muted-foreground hover:text-foreground" />
          </div>
        </TooltipTrigger>
        <TooltipContent side="bottom" className="w-64 p-0">
          <div className="flex items-center gap-2 border-b border-border px-3 py-2">
            <span className="text-xs font-medium">{t("Attributes")}</span>
            {fields.length > 0 && (
              <span className="rounded-full bg-muted px-1.5 py-px font-mono text-[10px] text-muted-foreground">
                {fields.length}
              </span>
            )}
          </div>
          {fields.length === 0 ? (
            <p className="px-3 py-2 text-xs text-muted-foreground">
              {t("No attributes detected.")}
            </p>
          ) : (
            <div className="flex max-h-56 flex-col overflow-y-auto p-1">
              {fields.map((field) => {
                return (
                  <div
                    key={field.name}
                    className="flex items-center gap-2 rounded px-2 py-1 hover:bg-accent">
                    <code className="flex-1 truncate font-mono text-xs">
                      {field.name}
                    </code>
                    <span
                      className={`font-mono text-[10px] font-semibold ${TYPE_COLOR[field.type]}`}>
                      {field.type}
                    </span>
                  </div>
                );
              })}
            </div>
          )}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
};

export default memo(SchemaIndicator);
