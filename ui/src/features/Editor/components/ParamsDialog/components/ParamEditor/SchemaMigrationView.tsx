import { ArrowRightIcon, WarningCircleIcon } from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { useMemo, useState } from "react";

import { Alert, AlertDescription, Button, SchemaForm } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { AwarenessUser, NodeParams } from "@flow/types";

import { FieldContext } from "../../utils/fieldUtils";

type Props = {
  readonly?: boolean;
  storedSchema?: RJSFSchema;
  storedParams?: NodeParams;
  newSchema?: RJSFSchema;
  actionName?: string;
  fieldFocusMap?: Record<string, AwarenessUser[]>;
  onParamFieldFocus?: (fieldId: string | null) => void;
  onMigrate: (newParams: NodeParams) => void;
  onValueEditorOpen: (fieldContext: FieldContext) => void;
  onPythonEditorOpen?: (fieldContext: FieldContext) => void;
};

const SchemaMigrationView: React.FC<Props> = ({
  readonly,
  storedSchema,
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
  const [formKey, setFormKey] = useState(0);

  const migratedInitialData = useMemo(() => {
    if (!newSchema?.properties || !storedParams) return {};
    const newKeys = new Set(Object.keys(newSchema.properties));
    return Object.fromEntries(
      Object.entries(storedParams).filter(([key]) => newKeys.has(key)),
    );
  }, [newSchema, storedParams]);

  const [migrationData, setMigrationData] =
    useState<NodeParams>(migratedInitialData);

  const handleStartFromScratch = () => {
    setMigrationData({});
    setFormKey((k) => k + 1);
  };

  const handleSubmit = () => {
    if (!isValid) return;
    onMigrate(migrationData);
  };

  return (
    <div className="flex size-full flex-col gap-3 p-2">
      <Alert className="shrink-0 border-yellow-500/40 bg-yellow-500/10 py-2 text-yellow-700 dark:text-yellow-400 [&>svg]:top-3">
        <div className="flex items-center gap-2">
          <WarningCircleIcon className="size-4" />
          <AlertDescription className="text-xs">
            {t(
              "The action has potentially been updated. Please migrate your settings to the new action schema parameters.",
            )}
          </AlertDescription>
        </div>
      </Alert>

      <div className="flex min-h-0 flex-1 items-stretch gap-3">
        <div className="flex min-w-0 flex-1 flex-col gap-2 rounded border border-border/50 p-3">
          <p className="shrink-0 text-xs font-medium tracking-wide text-muted-foreground uppercase">
            {t("Previous Parameters")}
          </p>
          <div className="min-h-0 flex-1 overflow-y-auto opacity-75">
            {storedSchema ? (
              <SchemaForm
                readonly
                schema={storedSchema}
                defaultFormData={storedParams}
                onChange={() => {}}
              />
            ) : storedParams && Object.keys(storedParams).length > 0 ? (
              <div className="space-y-1 pt-1">
                {Object.entries(storedParams).map(([key, value]) => (
                  <div key={key} className="flex gap-2 text-xs">
                    <span className="font-medium">{key}:</span>
                    <span className="break-all text-muted-foreground">
                      {JSON.stringify(value)}
                    </span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="pt-4 text-xs text-muted-foreground italic">
                {t("No previous settings available")}
              </p>
            )}
          </div>
        </div>

        <div className="flex shrink-0 items-center">
          <ArrowRightIcon className="size-6 text-green-500" weight="bold" />
        </div>

        <div className="flex min-w-0 flex-1 flex-col gap-2 rounded border border-border/50 p-3">
          <p className="shrink-0 text-xs font-medium tracking-wide text-muted-foreground uppercase">
            {t("New Parameters")}
          </p>
          <div className="min-h-0 flex-1 overflow-y-auto">
            <SchemaForm
              key={formKey}
              readonly={readonly}
              schema={newSchema}
              actionName={actionName}
              defaultFormData={migrationData}
              fieldFocusMap={fieldFocusMap}
              onFieldFocus={onParamFieldFocus}
              onChange={(data) => setMigrationData(data ?? {})}
              onValidationChange={setIsValid}
              onEditorOpen={onValueEditorOpen}
              onPythonEditorOpen={onPythonEditorOpen}
            />
          </div>
        </div>
      </div>

      <div className="flex shrink-0 justify-end gap-2">
        <Button
          variant="outline"
          size="lg"
          className="border-orange-500/60 text-orange-600 hover:bg-orange-500/10 hover:text-orange-600 dark:text-orange-400"
          onClick={handleStartFromScratch}
          disabled={readonly}>
          {t("Start from scratch")}
        </Button>
        <Button
          size="lg"
          onClick={handleSubmit}
          disabled={readonly || !isValid}>
          {t("Submit")}
        </Button>
      </div>
    </div>
  );
};

export default SchemaMigrationView;
