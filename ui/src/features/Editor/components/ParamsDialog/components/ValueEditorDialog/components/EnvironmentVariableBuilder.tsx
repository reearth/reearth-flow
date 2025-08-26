import { CircleIcon, CheckIcon } from "@phosphor-icons/react";
import { useCallback, useState, useEffect } from "react";

import {
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@flow/components";
import { useProjectVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentProject } from "@flow/stores";

import ExpressionInput from "./ExpressionInput";

type VariableAccessType =
  | "project_variable"
  | "custom_variable"
  | "workflow_parameter";

type Props = {
  onExpressionChange: (expression: string) => void;
};

const EnvironmentVariableBuilder: React.FC<Props> = ({
  onExpressionChange,
}) => {
  const t = useT();
  const [currentProject] = useCurrentProject();

  const [accessType, setAccessType] =
    useState<VariableAccessType>("project_variable");
  const [selectedVariable, setSelectedVariable] = useState("");
  const [customVariableName, setCustomVariableName] = useState("");

  const { useGetProjectVariables } = useProjectVariables();
  const { projectVariables } = useGetProjectVariables(currentProject?.id);

  const accessTypes = [
    {
      value: "project_variable" as const,
      label: t("Project Variables"),
      description: t("Access variables defined in the project"),
      icon: <CircleIcon weight="thin" className="h-4 w-4" />,
      example: 'env.get("outputPath")',
    },
    {
      value: "custom_variable" as const,
      label: t("Custom Variable"),
      description: t("Access any environment variable by name"),
      icon: <CircleIcon weight="thin" className="h-4 w-4" />,
      example: 'env.get("CUSTOM_VAR")',
    },
    {
      value: "workflow_parameter" as const,
      label: t("Workflow Parameters"),
      description: t("Access common workflow parameters"),
      icon: <CircleIcon weight="thin" className="h-4 w-4" />,
      example: 'env.get("__workflow_id")',
    },
  ];

  // Common workflow parameters
  const workflowParameters = [
    "__workflow_id",
    "__project_id",
    "__workspace_id",
    "__user_id",
    "__timestamp",
    "__batch_id",
    "__execution_id",
  ];

  const generateExpression = useCallback(() => {
    let expr = "";

    switch (accessType) {
      case "project_variable":
        if (selectedVariable) {
          expr = `env.get("${selectedVariable}")`;
        }
        break;
      case "custom_variable":
        if (customVariableName) {
          expr = `env.get("${customVariableName}")`;
        }
        break;
      case "workflow_parameter":
        if (selectedVariable) {
          expr = `env.get("${selectedVariable}")`;
        }
        break;
    }

    onExpressionChange(expr);
  }, [accessType, selectedVariable, customVariableName, onExpressionChange]);

  // Generate expression whenever inputs change
  useEffect(() => {
    generateExpression();
  }, [generateExpression]);

  const selectedAccessType = accessTypes.find(
    (type) => type.value === accessType,
  );

  return (
    <div className="flex size-full flex-col gap-4 p-4">
      <div className="flex-shrink-0">
        <h4 className="text-sm font-medium">{t("Environment Variables")}</h4>
        <p className="text-xs text-muted-foreground">
          {t("Access project variables and workflow parameters")}
        </p>
      </div>

      <div className="space-y-4">
        {/* Two-column layout for source selection and example */}
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="access-type-select" className="text-xs">
              {t("Variable Source")}
            </Label>
            <Select
              value={accessType}
              onValueChange={(value) => {
                setAccessType(value as VariableAccessType);
                setSelectedVariable("");
                setCustomVariableName("");
              }}>
              <SelectTrigger id="access-type-select">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {accessTypes.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    <div className="flex items-center gap-2">
                      {type.icon}
                      <div>
                        <div className="text-sm">{type.label}</div>
                        <div className="text-xs text-muted-foreground">
                          {type.description}
                        </div>
                      </div>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {selectedAccessType && (
            <div className="rounded border bg-muted/30 p-3">
              <div className="mb-2 flex items-center gap-2 text-xs text-muted-foreground">
                {selectedAccessType.icon}
                <span>{t("Example:")} </span>
              </div>
              <code className="text-xs break-all">{selectedAccessType.example}</code>
            </div>
          )}
        </div>

        <div className="space-y-4">
          {accessType === "project_variable" && (
            <div className="space-y-3">
              <Label className="text-xs">{t("Project Variables")}</Label>
              {projectVariables && projectVariables.length > 0 ? (
                <div className="grid grid-cols-1 gap-2 lg:grid-cols-2 xl:grid-cols-3">
                  {projectVariables.map((variable) => (
                    <div
                      key={variable.id}
                      className={`flex cursor-pointer flex-col rounded border p-3 transition-colors ${
                        selectedVariable === variable.name
                          ? "border-accent-foreground/20 bg-accent"
                          : "hover:bg-accent/50"
                      }`}
                      onClick={() => setSelectedVariable(variable.name)}>
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1 min-w-0">
                          <div className="text-sm font-medium truncate">
                            {variable.name}
                          </div>
                          <div className="text-xs text-muted-foreground">
                            {variable.type}
                          </div>
                        </div>
                        {selectedVariable === variable.name && (
                          <CheckIcon className="h-4 w-4 text-accent-foreground flex-shrink-0" />
                        )}
                      </div>
                      <div className="mt-1 text-xs text-muted-foreground truncate">
                        {variable.defaultValue || t("No value set")}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="py-8 text-center text-sm text-muted-foreground">
                  {t("No project variables found")}
                </div>
              )}
            </div>
          )}

          {accessType === "custom_variable" && (
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
              <div className="space-y-2">
                <Label htmlFor="custom-variable-name" className="text-xs">
                  {t("Variable Name")}
                </Label>
                <ExpressionInput
                  placeholder="CUSTOM_VAR"
                  value={customVariableName}
                  onChange={setCustomVariableName}
                  className="font-mono text-sm"
                  label={t("Variable Name")}
                  allowedExpressionTypes={["feature-attribute", "math"]}
                />
              </div>
              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground">{t("Usage Notes")}</Label>
                <div className="rounded border bg-muted/30 p-3 text-xs text-muted-foreground">
                  <div className="space-y-1">
                    <div>• {t("Enter the exact name of the environment variable")}</div>
                    <div>• {t("Variable names are case-sensitive")}</div>
                    <div>• {t("Examples: API_KEY, DB_HOST, VERSION")}</div>
                  </div>
                </div>
              </div>
            </div>
          )}

          {accessType === "workflow_parameter" && (
            <div className="space-y-3">
              <Label className="text-xs">{t("Workflow Parameters")}</Label>
              <div className="grid grid-cols-1 gap-2 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                {workflowParameters.map((param) => (
                  <div
                    key={param}
                    className={`flex cursor-pointer items-center justify-between rounded border p-2 transition-colors ${
                      selectedVariable === param
                        ? "border-accent-foreground/20 bg-accent"
                        : "hover:bg-accent/50"
                    }`}
                    onClick={() => setSelectedVariable(param)}>
                    <code className="text-sm flex-1 truncate">{param}</code>
                    {selectedVariable === param && (
                      <CheckIcon className="h-4 w-4 text-accent-foreground flex-shrink-0 ml-2" />
                    )}
                  </div>
                ))}
              </div>
              <div className="text-xs text-muted-foreground">
                {t(
                  "These parameters are automatically available during workflow execution",
                )}
              </div>
            </div>
          )}
        </div>

      </div>
    </div>
  );
};

export default EnvironmentVariableBuilder;
