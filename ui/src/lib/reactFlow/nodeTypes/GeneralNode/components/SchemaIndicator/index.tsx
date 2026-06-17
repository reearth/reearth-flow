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
import { useT } from "@flow/lib/i18n";
import { useReaderSchemaProbes } from "@flow/stores";
import type { FieldReport, NodeSchemaMeta } from "@flow/types";

type Props = {
  nodeId: string;
  schema?: NodeSchemaMeta;
};

/** Flatten a node's ports into a deduped, ordered list of attribute fields. */
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

/**
 * Bottom-left indicator on a (reader) node reflecting the attribute-schema
 * probe: a spinner while probing, a warning on failure, and an info icon that
 * reveals the probed attributes on hover once available.
 */
const SchemaIndicator: React.FC<Props> = ({ nodeId, schema }) => {
  const t = useT();
  const [probes] = useReaderSchemaProbes();
  const probe = probes[nodeId];

  const fields = useMemo(() => collectFields(schema), [schema]);

  if (probe?.status === "running") {
    return (
      <div className="absolute bottom-1 left-1 flex items-center justify-center">
        <CircleNotchIcon className="size-3 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (probe?.status === "failed") {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="absolute bottom-1 left-1 flex items-center justify-center">
              <WarningCircleIcon className="size-3 text-warning" />
            </div>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            {t("Schema preview failed. Re-save to retry.")}
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
          <div className="absolute bottom-1 left-1 flex items-center justify-center">
            <InfoIcon className="size-3 text-muted-foreground hover:text-foreground" />
          </div>
        </TooltipTrigger>
        <TooltipContent side="bottom" className="max-w-xs">
          <div className="flex flex-col gap-1">
            <p className="font-medium">{t("Attributes")}</p>
            {fields.length === 0 ? (
              <p className="text-muted-foreground">
                {t("No attributes detected.")}
              </p>
            ) : (
              <div className="flex flex-col gap-0.5">
                {fields.map((field) => (
                  <code key={field.name} className="text-xs">
                    {field.name}
                    {field.presence === "maybe" ? "?" : ""} :{" "}
                    <span className="text-muted-foreground">{field.type}</span>
                  </code>
                ))}
              </div>
            )}
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
};

export default memo(SchemaIndicator);
