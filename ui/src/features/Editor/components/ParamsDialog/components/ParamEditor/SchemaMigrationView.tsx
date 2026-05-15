import {
  ArrowRightIcon,
  CaretDownIcon,
  CaretRightIcon,
  WarningCircleIcon,
} from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { useMemo, useState } from "react";

import {
  Alert,
  AlertDescription,
  AlertTitle,
  Button,
  SchemaForm,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";
import type { AwarenessUser, NodeParams } from "@flow/types";

import { FieldContext } from "../../utils/fieldUtils";

type Props = {
  readonly?: boolean;
  storedParams?: NodeParams;
  newSchema?: RJSFSchema;
  actionName?: string;
  fieldFocusMap?: Record<string, AwarenessUser[]>;
  onParamFieldFocus?: (fieldId: string | null) => void;
  onMigrate: (newParams: NodeParams) => void;
  onValueEditorOpen: (fieldContext: FieldContext) => void;
  onPythonEditorOpen?: (fieldContext: FieldContext) => void;
};

type NestedValueProps = {
  value: unknown;
  depth?: number;
};

const NestedValue: React.FC<NestedValueProps> = ({ value, depth = 0 }) => {
  const [expanded, setExpanded] = useState(false);

  if (value === null || value === undefined) {
    return <span className="text-muted-foreground italic">null</span>;
  }
  if (typeof value !== "object") {
    return <span className="break-all text-foreground">{String(value)}</span>;
  }

  const isArray = Array.isArray(value);
  const entries: [string, unknown][] = isArray
    ? (value as unknown[]).map((v, i) => [String(i), v])
    : Object.entries(value as Record<string, unknown>);
  const summary = isArray
    ? `[${entries.length} item${entries.length !== 1 ? "s" : ""}]`
    : `{${entries.length} key${entries.length !== 1 ? "s" : ""}}`;

  return (
    <div>
      <button
        type="button"
        onClick={() => setExpanded((v) => !v)}
        className="flex items-center gap-0.5 text-muted-foreground transition-colors hover:text-foreground">
        {expanded ? (
          <CaretDownIcon className="size-3 shrink-0" />
        ) : (
          <CaretRightIcon className="size-3 shrink-0" />
        )}
        <span className="text-xs italic">{summary}</span>
      </button>
      {expanded && (
        <div
          className={cn(
            "mt-1 space-y-0.5 border-l pl-3",
            depth === 0 ? "border-border" : "border-border/50",
          )}>
          {entries.map(([k, v]) => (
            <div key={k} className="flex min-w-0 gap-1.5">
              <span className="shrink-0 text-muted-foreground">{k}:</span>
              <NestedValue value={v} depth={depth + 1} />
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

const ParamRow: React.FC<{ paramKey: string; value: unknown }> = ({
  paramKey,
  value,
}) => {
  const isPrimitive =
    value === null || value === undefined || typeof value !== "object";

  return (
    <div className="flex min-w-0 gap-1.5 py-0.5">
      <span className="shrink-0 font-medium text-foreground/80">
        {paramKey}:
      </span>
      {isPrimitive ? (
        <span className="break-all text-foreground">
          {String(value ?? "null")}
        </span>
      ) : (
        <NestedValue value={value} />
      )}
    </div>
  );
};

const SchemaMigrationView: React.FC<Props> = ({
  readonly,
  storedParams,
  newSchema,
  actionName,
  fieldFocusMap,
  onParamFieldFocus,
  onMigrate,
  onValueEditorOpen,
  onPythonEditorOpen,
}) => {
  const t = useT();
  const [isValid, setIsValid] = useState(true);

  const migratedInitialData = useMemo(() => {
    if (!newSchema?.properties || !storedParams) return {};
    const newKeys = new Set(Object.keys(newSchema.properties));
    return Object.fromEntries(
      Object.entries(storedParams).filter(([key]) => newKeys.has(key)),
    );
  }, [newSchema, storedParams]);

  const [migrationData, setMigrationData] =
    useState<NodeParams>(migratedInitialData);

  const handleFormChange = (data: any) => {
    setMigrationData(data ?? {});
  };

  const handleSubmit = () => {
    if (!isValid) return;
    onMigrate(migrationData);
  };

  const paramEntries = storedParams ? Object.entries(storedParams) : [];

  return (
    <div className="flex size-full min-h-0 flex-col gap-3">
      <Alert className="shrink-0 border-yellow-500/40 bg-yellow-500/10 py-3 text-yellow-700 dark:text-yellow-400">
        <WarningCircleIcon className="size-4" />
        <AlertTitle className="text-sm">
          {t("Action Schema Updated")}
        </AlertTitle>
        <AlertDescription className="text-xs opacity-80">
          {t(
            "The action has potentially been updated. Please migrate your settings to the new action schema.",
          )}
        </AlertDescription>
      </Alert>

      <div className="flex min-h-0 flex-1 gap-3">
        <div className="flex w-[42%] flex-col gap-1.5">
          <p className="shrink-0 text-xs font-semibold tracking-wide text-muted-foreground uppercase">
            {t("Previous Values")}
          </p>
          <div className="min-h-0 flex-1 overflow-y-auto rounded border bg-muted/20 p-3 font-mono text-xs">
            {paramEntries.length > 0 ? (
              <div className="space-y-1">
                {paramEntries.map(([key, value]) => (
                  <ParamRow key={key} paramKey={key} value={value} />
                ))}
              </div>
            ) : (
              <span className="text-muted-foreground italic">
                {t("No previous values")}
              </span>
            )}
          </div>
        </div>

        <div className="flex shrink-0 items-center justify-center">
          <ArrowRightIcon className="size-4 text-muted-foreground" />
        </div>

        <div className="flex flex-1 flex-col justify-between gap-3">
          <div className="min-h-0 flex-1 overflow-y-auto rounded px-1">
            <SchemaForm
              readonly={readonly}
              schema={newSchema}
              actionName={actionName}
              defaultFormData={migrationData}
              fieldFocusMap={fieldFocusMap}
              onFieldFocus={onParamFieldFocus}
              onChange={handleFormChange}
              onValidationChange={setIsValid}
              onEditorOpen={onValueEditorOpen}
              onPythonEditorOpen={onPythonEditorOpen}
            />
          </div>
          <Button
            className="shrink-0 self-end"
            size="lg"
            onClick={handleSubmit}
            disabled={readonly || !isValid}>
            {t("Submit")}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default SchemaMigrationView;
