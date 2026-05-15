import {
  ArrowCircleRightIcon,
  ClockCounterClockwiseIcon,
  WarningCircleIcon,
} from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { useMemo, useState } from "react";

import {
  Alert,
  AlertDescription,
  Button,
  SchemaForm,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
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

  const migratedInitialData = useMemo(() => {
    if (!newSchema?.properties || !storedParams) return {};
    const newKeys = new Set(Object.keys(newSchema.properties));
    return Object.fromEntries(
      Object.entries(storedParams).filter(([key]) => newKeys.has(key)),
    );
  }, [newSchema, storedParams]);

  const [migrationData, setMigrationData] =
    useState<NodeParams>(migratedInitialData);

  const handleSubmit = () => {
    if (!isValid) return;
    onMigrate(migrationData);
  };

  return (
    <div className="flex size-full flex-col gap-2">
      <div className="p-2">
        <Alert className="flex shrink-0 items-center gap-2 border-yellow-500/40 bg-yellow-500/10 py-2 text-yellow-700 dark:text-yellow-400 [&>svg]:top-3">
          <div>
            <WarningCircleIcon className="size-4" />
          </div>
          <AlertDescription className="text-xs">
            {t(
              "The action has potentially been updated. Please migrate your settings to the new action schema.",
            )}
          </AlertDescription>
        </Alert>
      </div>

      <Tabs defaultValue="updated" className="flex min-h-0 flex-1">
        <TabsList className="flex h-full w-50 flex-col justify-start gap-2 rounded-none p-2">
          <TabsTrigger
            className="h-7.5 w-full justify-start gap-2"
            value="previous">
            <ClockCounterClockwiseIcon className="shrink-0" />
            <p>{t("Previous Version")}</p>
          </TabsTrigger>
          <TabsTrigger
            className="h-7.5 w-full justify-start gap-2"
            value="updated">
            <ArrowCircleRightIcon className="shrink-0" />
            <p>{t("Updated Version")}</p>
          </TabsTrigger>
        </TabsList>
        <div className="h-full self-center border-r dark:border-primary" />

        <TabsContent className="px-6 py-4" value="previous" asChild>
          <div className="flex size-full min-h-0 flex-col gap-4">
            <div className="min-h-0 flex-1 overflow-y-auto rounded px-2 pt-1 opacity-75">
              {storedSchema ? (
                <SchemaForm
                  readonly
                  schema={storedSchema}
                  defaultFormData={storedParams}
                  onChange={() => {}}
                />
              ) : (
                <p className="pt-4 text-xs text-muted-foreground italic">
                  {t("No previous schema available")}
                </p>
              )}
            </div>
          </div>
        </TabsContent>

        <TabsContent className="px-6 py-4" value="updated" asChild>
          <div className="flex size-full min-h-0 flex-col justify-between gap-4">
            <div className="min-h-0 flex-1 overflow-y-auto rounded px-2 pt-1">
              <SchemaForm
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
            <Button
              className="shrink-0 self-end"
              size="lg"
              onClick={handleSubmit}
              disabled={readonly || !isValid}>
              {t("Submit")}
            </Button>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default SchemaMigrationView;
