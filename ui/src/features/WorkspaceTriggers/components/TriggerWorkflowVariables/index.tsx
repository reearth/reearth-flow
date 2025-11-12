import { useState } from "react";

import {
  Button,
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
import {
  getDefaultValue,
  inferProjectVariableType,
} from "@flow/features/WorkspaceProjects/components/WorkflowImport/inferVariableType";
import SimpleArrayInput from "@flow/features/WorkspaceProjects/components/WorkflowImport/SimpleArrayInput";
import { useT } from "@flow/lib/i18n";
import { VarType } from "@flow/types";
import type { WorkflowVariable } from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

type VariableMapping = {
  name: string;
  type: VarType;
  defaultValue: any;
};

type TriggerProjectVariablesMappingDialogProps = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  variables: WorkflowVariable[] | Record<string, any>[];
  workflowName: string;
  onConfirm: (projectVariables: any[]) => void;
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
        };
      }),
  );

  const handleDefaultValueChange = (index: number, newValue: any) => {
    setVariableMappings((prev) =>
      prev.map((mapping, i) =>
        i === index ? { ...mapping, defaultValue: newValue } : mapping,
      ),
    );
  };
  const handleConfirm = () => {
    const projectVariables = variableMappings.map((mapping) => ({
      name: mapping.name,
      type: mapping.type,
      defaultValue: mapping.defaultValue,
    }));

    onConfirm(projectVariables);
  };

  const handleCancel = () => {
    onCancel();
    onOpenChange(false);
  };

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent size="xl">
        <DialogHeader>
          <DialogTitle>{t("Configure Project Variables")}</DialogTitle>
          <DialogDescription>
            {t(
              "The deployment contains {{count}} variables. Configure how they should be set as Project Variables.",
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
              </div>

              <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div className="flex flex-col justify-center space-y-2">
                  <Label htmlFor={`type-${index}`}>{t("Variable Type")}</Label>
                  <span className="flex h-8 w-fit items-center gap-1 rounded-md bg-primary px-2 py-0.5 text-sm">
                    {mapping.type}
                  </span>
                </div>
                <div className="space-y-2">
                  <Label htmlFor={`default-${index}`}>
                    <span>{t("Default Value")}</span>
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
        <DialogFooter className="mt-2">
          <Button variant="outline" onClick={handleCancel}>
            {t("Cancel")}
          </Button>
          <Button onClick={handleConfirm}>{t("Confirm Variables")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
