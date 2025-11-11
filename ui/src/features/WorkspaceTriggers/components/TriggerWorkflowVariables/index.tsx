import { useState } from "react";

import {
  Button,
  Checkbox,
  Dialog,
  DialogContent,
  DialogContentWrapper,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  Label,
  TextArea,
} from "@flow/components";
import { VariableTypeSelector } from "@flow/features/WorkspaceProjects/components";
import {
  getDefaultValue,
  inferProjectVariableType,
} from "@flow/features/WorkspaceProjects/components/WorkflowImport/inferVariableType";
import SimpleArrayInput from "@flow/features/WorkspaceProjects/components/WorkflowImport/SimpleArrayInput";
import { useT } from "@flow/lib/i18n";
import { AnyProjectVariable, VarType } from "@flow/types";
import type { WorkflowVariable } from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

type VariableMapping = {
  name: string;
  type: VarType;
  defaultValue: any;
  required: boolean;
  public: boolean;
};

type TriggerProjectVariablesMappingDialogProps = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  variables: WorkflowVariable[];
  workflowName: string;
  onConfirm: (
    projectVariables: Omit<
      AnyProjectVariable,
      "id" | "createdAt" | "updatedAt" | "projectId"
    >[],
  ) => void;
  onCancel: () => void;
};

export default function TriggerProjectVariablesMappingDialog({
  isOpen,
  onOpenChange,
  variables,
  workflowName,
  onConfirm,
  onCancel,
}: TriggerProjectVariablesMappingDialogProps) {
  const t = useT();

  // Initialize variable mappings with inferred types
  const [variableMappings, setVariableMappings] = useState<VariableMapping[]>(
    () =>
      variables.map((variable) => {
        const inferredType = inferProjectVariableType(
          variable.value,
          variable.name,
        );
        return {
          name: variable.name,
          type: inferredType,
          defaultValue: getDefaultValue(variable.value, inferredType),
          required: variable.value !== null && variable.value !== undefined,
          public: false,
        };
      }),
  );

  const handleTypeChange = (index: number, newType: VarType) => {
    setVariableMappings((prev) =>
      prev.map((mapping, i) =>
        i === index
          ? {
              ...mapping,
              type: newType,
              defaultValue: getDefaultValue(mapping.defaultValue, newType),
            }
          : mapping,
      ),
    );
  };

  const handleDefaultValueChange = (index: number, newValue: any) => {
    setVariableMappings((prev) =>
      prev.map((mapping, i) =>
        i === index ? { ...mapping, defaultValue: newValue } : mapping,
      ),
    );
  };

  const handleRequiredChange = (index: number, required: boolean) => {
    setVariableMappings((prev) =>
      prev.map((mapping, i) =>
        i === index ? { ...mapping, required } : mapping,
      ),
    );
  };

  const handlePublicChange = (index: number, isPublic: boolean) => {
    setVariableMappings((prev) =>
      prev.map((mapping, i) =>
        i === index ? { ...mapping, public: isPublic } : mapping,
      ),
    );
  };

  const handleConfirm = () => {
    const projectVariables = variableMappings.map((mapping) => ({
      name: mapping.name,
      type: mapping.type,
      defaultValue: mapping.defaultValue,
      required: mapping.required,
      public: mapping.public,
      config: undefined, // Basic implementation without specific config
    }));

    onConfirm(projectVariables);
  };

  const handleCancel = () => {
    onCancel();
    onOpenChange(false);
  };

  // const renderValuePreview = (value: any) => {
  //   if (value === null || value === undefined) {
  //     return <span className="text-muted-foreground italic">null</span>;
  //   }
  //   if (Array.isArray(value)) {
  //     return <span className="font-mono text-xs">[{value.join(", ")}]</span>;
  //   }
  //   if (typeof value === "object") {
  //     return <span className="font-mono text-xs">{JSON.stringify(value)}</span>;
  //   }
  //   return <span className="font-mono text-xs">{String(value)}</span>;
  // };

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent size="xl">
        <DialogHeader>
          <DialogTitle>{t("Configure Workflow Variables")}</DialogTitle>
          <DialogDescription>
            {t(
              "The workflow '{{workflowName}}' contains {{count}} variables. Configure how they should be imported as Project Variables.",
              {
                workflowName,
                count: variables.length,
              },
            )}
          </DialogDescription>
        </DialogHeader>
        <DialogContentWrapper className="h-[400px] overflow-auto">
          {variableMappings.map((mapping, index) => (
            <div key={mapping.name} className="space-y-4 rounded-lg border p-4">
              <div className="flex items-start justify-between">
                <div className="space-y-1">
                  <Label className="text-sm font-semibold">
                    {mapping.name}
                  </Label>
                </div>
                <div className="flex items-center space-x-4">
                  <div className="flex items-center space-x-2">
                    <Checkbox
                      id={`required-${index}`}
                      checked={mapping.required}
                      onCheckedChange={(checked) =>
                        handleRequiredChange(index, checked as boolean)
                      }
                    />
                    <Label htmlFor={`required-${index}`} className="text-sm">
                      {t("Required")}
                    </Label>
                  </div>

                  <div className="flex items-center space-x-2">
                    <Checkbox
                      id={`public-${index}`}
                      checked={mapping.public}
                      onCheckedChange={(checked) =>
                        handlePublicChange(index, checked as boolean)
                      }
                    />
                    <Label htmlFor={`public-${index}`} className="text-sm">
                      {t("Public")}
                    </Label>
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor={`type-${index}`}>{t("Variable Type")}</Label>
                  <VariableTypeSelector
                    value={mapping.type}
                    onValueChange={(newType) =>
                      handleTypeChange(index, newType)
                    }
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor={`default-${index}`}>
                    {t("Default Value")}
                  </Label>
                  {mapping.type === "array" ? (
                    <SimpleArrayInput
                      value={
                        Array.isArray(mapping.defaultValue)
                          ? mapping.defaultValue
                          : []
                      }
                      onChange={(newValue) =>
                        handleDefaultValueChange(index, newValue)
                      }
                    />
                  ) : mapping.type === "text" &&
                    typeof mapping.defaultValue === "string" &&
                    mapping.defaultValue.length > 50 ? (
                    <TextArea
                      id={`default-${index}`}
                      value={mapping.defaultValue}
                      onChange={(e) =>
                        handleDefaultValueChange(index, e.target.value)
                      }
                      className="min-h-[60px]"
                    />
                  ) : (
                    <Input
                      id={`default-${index}`}
                      type={
                        mapping.type === "number"
                          ? "number"
                          : mapping.type === "password"
                            ? "password"
                            : "text"
                      }
                      value={mapping.defaultValue}
                      onChange={(e) => {
                        const value =
                          mapping.type === "number"
                            ? parseFloat(e.target.value) || 0
                            : e.target.value;
                        handleDefaultValueChange(index, value);
                      }}
                    />
                  )}
                </div>
              </div>
            </div>
          ))}
        </DialogContentWrapper>
        <DialogFooter className=" mt-2">
          <Button variant="outline" onClick={handleCancel}>
            {t("Cancel")}
          </Button>
          <Button onClick={handleConfirm}>{t("Confirm Variables")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
