import {
  ArrowUDownLeftIcon,
  ChalkboardTeacherIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useMemo, useState } from "react";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  IconButton,
  DataTable as Table,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  VariableRow,
} from "@flow/components";
import { Button } from "@flow/components/buttons/BaseButton";
import {
  getDefaultValue,
  inferWorkflowVariableType,
} from "@flow/features/WorkspaceProjects/components/WorkflowImport/inferVariableType";
import { useT } from "@flow/lib/i18n";
import { TriggerVariableMapping } from "@flow/types";
import { WorkflowVariable } from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

type TriggerWorkflowVariablesMappingDialogProps = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  variables: WorkflowVariable[] | Record<string, any>[];
  workflowName: string;
  deploymentDefaults?: Record<string, any>; // Optional deployment defaults for reset functionality
  onConfirm: (workflowVariables: any[]) => void;
  onCancel: () => void;
};

const TriggerWorkflowVariablesMappingDialog: React.FC<
  TriggerWorkflowVariablesMappingDialogProps
> = ({
  isOpen,
  onOpenChange,
  variables,
  workflowName,
  deploymentDefaults,
  onConfirm,
  onCancel,
}) => {
  const t = useT();

  const [variableMappings, setVariableMappings] = useState<
    TriggerVariableMapping[]
  >(() =>
    variables.map((variable) => {
      const inferredType = inferWorkflowVariableType(
        variable.value,
        variable.name,
      );
      const deploymentDefault = deploymentDefaults?.[variable.name];
      return {
        name: variable.name,
        type: inferredType,
        defaultValue: getDefaultValue(variable.value, inferredType),
        deploymentDefault: deploymentDefault,
      };
    }),
  );

  const handleDefaultValueChange = useCallback(
    (index: number, newValue: any) => {
      setVariableMappings((prev) =>
        prev.map((mapping, i) =>
          i === index ? { ...mapping, defaultValue: newValue } : mapping,
        ),
      );
    },
    [],
  );

  const handleResetToDefault = useCallback((index: number) => {
    setVariableMappings((prev) =>
      prev.map((mapping, i) => {
        if (i === index && mapping.deploymentDefault !== undefined) {
          return { ...mapping, defaultValue: mapping.deploymentDefault };
        }
        return mapping;
      }),
    );
  }, []);

  const isAtDefault = useCallback(
    (mapping: TriggerVariableMapping): boolean => {
      if (mapping.deploymentDefault === undefined) return true;
      return (
        JSON.stringify(mapping.defaultValue) ===
        JSON.stringify(mapping.deploymentDefault)
      );
    },
    [],
  );

  const handleConfirm = () => {
    const workflowVariables = variableMappings.map((mapping) => ({
      name: mapping.name,
      type: mapping.type,
      defaultValue: mapping.defaultValue,
    }));

    onConfirm(workflowVariables);
  };

  const handleCancel = () => {
    onCancel();
    onOpenChange(false);
  };

  const columns: ColumnDef<TriggerVariableMapping>[] = useMemo(
    () => [
      {
        accessorKey: "name",
        header: t("Name"),
      },
      {
        accessorKey: "type",
        header: t("Type"),
      },
      {
        accessorKey: "defaultValue",
        header: t("Default Value"),
        cell: ({ row }) => {
          return (
            <VariableRow
              variable={row.original}
              index={row.index}
              onDefaultValueChange={handleDefaultValueChange}
            />
          );
        },
      },
      {
        id: "actions",
        header: t("Actions"),
        cell: ({ row }) => {
          return (
            <div className="flex items-center gap-1">
              {row.original.deploymentDefault !== undefined && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <IconButton
                      size="sm"
                      variant="ghost"
                      icon={<ArrowUDownLeftIcon />}
                      onClick={() => handleResetToDefault(row.index)}
                      disabled={isAtDefault(row.original)}
                    />
                  </TooltipTrigger>
                  <TooltipContent>
                    {t("Reset to workflow default")}
                  </TooltipContent>
                </Tooltip>
              )}
            </div>
          );
        },
        size: 100,
      },
    ],
    [handleDefaultValueChange, handleResetToDefault, isAtDefault, t],
  );

  return (
    <Dialog open={isOpen} onOpenChange={handleCancel}>
      <DialogContent
        className="h-[50vh] focus-visible:ring-0 focus-visible:outline-none"
        size="2xl"
        position="off-center">
        <div className="flex h-full flex-col">
          <DialogHeader>
            <DialogTitle>
              <div className="flex items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <ChalkboardTeacherIcon />
                  {`${workflowName} ${t("Workflow Variables")}`}
                </div>
              </div>
            </DialogTitle>
          </DialogHeader>
          <div className="flex h-full min-h-0">
            <DialogContentSection className="flex min-h-0 flex-3 flex-col">
              <DialogContentSection className="mt-4 min-h-0 flex-1 overflow-hidden">
                <Table
                  columns={columns}
                  data={variableMappings}
                  showOrdering={false}
                />
              </DialogContentSection>
            </DialogContentSection>
          </div>
          <DialogFooter className="flex justify-end gap-2 p-4">
            <Button variant="outline" onClick={handleCancel}>
              {t("Cancel")}
            </Button>
            <Button onClick={handleConfirm}>{t("Apply")}</Button>
          </DialogFooter>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default TriggerWorkflowVariablesMappingDialog;
