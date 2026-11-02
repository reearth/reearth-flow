import {
  ArrowUDownLeftIcon,
  ChalkboardTeacherIcon,
  TrashIcon,
} from "@phosphor-icons/react";
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
import AssetsDialog from "@flow/features/AssetsDialog";
import CmsIntegrationDialog from "@flow/features/CmsIntegrationDialog";
import {
  getDefaultValue,
  inferWorkflowVariableType,
} from "@flow/features/WorkspaceProjects/components/WorkflowImport/inferVariableType";
import { useT } from "@flow/lib/i18n";
import type { AppColumnDef } from "@flow/lib/table/features";
import type {
  AnyWorkflowVariable,
  Asset,
  TriggerVariableConfig,
} from "@flow/types";
import type { WorkflowVariable } from "@flow/utils/fromEngineWorkflow/deconstructedEngineWorkflow";

import { isEmptyVariableValue, toTypedMappings } from "./triggerVariables";

type TriggerWorkflowVariablesMappingDialogProps = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
  variables: WorkflowVariable[] | Record<string, any>[];
  workflowName: string;
  deploymentDefaults?: Record<string, any>; // Optional deployment defaults for reset functionality
  // The deployment's saved definitions. Without them, as for a deployment
  // uploaded as a file, each variable's type is guessed from its value.
  definitions?: AnyWorkflowVariable[];
  onConfirm: (workflowVariables: any[]) => void;
  onCancel: () => void;
};

type DialogOptions = "assets" | "cms" | undefined;

// A required variable left without a value: runs would be refused. A cleared
// field falls back to the deployment's default.
const isMissing = (mapping: TriggerVariableConfig) =>
  !!mapping.required &&
  isEmptyVariableValue(mapping.defaultValue ?? mapping.deploymentDefault);

const TriggerWorkflowVariablesMappingDialog: React.FC<
  TriggerWorkflowVariablesMappingDialogProps
> = ({
  isOpen,
  onOpenChange,
  variables,
  workflowName,
  deploymentDefaults,
  definitions,
  onConfirm,
  onCancel,
}) => {
  const t = useT();

  const [variableMappings, setVariableMappings] = useState<
    TriggerVariableConfig[]
  >(() => {
    const inferred = (name: string, value: any): TriggerVariableConfig => {
      const inferredType = inferWorkflowVariableType(value, name);
      return {
        name,
        type: inferredType,
        defaultValue: getDefaultValue(value, inferredType),
        deploymentDefault: deploymentDefaults?.[name],
      };
    };
    if (definitions) {
      return toTypedMappings(
        definitions,
        Object.fromEntries(variables.map((v) => [v.name, v.value])),
        inferred,
      );
    }
    return variables.map((variable) => inferred(variable.name, variable.value));
  });

  const missingCount = variableMappings.filter(isMissing).length;

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

  const handleRemove = useCallback((index: number) => {
    setVariableMappings((prev) => prev.filter((_, i) => i !== index));
  }, []);

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

  const isAtDefault = useCallback((mapping: TriggerVariableConfig): boolean => {
    if (mapping.deploymentDefault === undefined) return true;
    return (
      JSON.stringify(mapping.defaultValue) ===
      JSON.stringify(mapping.deploymentDefault)
    );
  }, []);

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

  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const [activeVariableIndex, setActiveVariableIndex] = useState<number>(0);
  const [activeArrayItemIndex, setActiveArrayItemIndex] = useState<number>(0);
  const [showVariableDialog, setShowVariableDialog] = useState(false);

  const handleDialogOpen = (dialog: DialogOptions) => {
    setShowDialog(dialog);
  };
  const handleDialogClose = () => setShowDialog(undefined);
  const handleVariableDialogOpen = useCallback(
    (variableIndex: number, arrayItemIndex = 0) => {
      setActiveVariableIndex(variableIndex);
      setActiveArrayItemIndex(arrayItemIndex);
      setShowVariableDialog(true);
    },
    [],
  );
  const handleVariableDialogClose = useCallback(
    () => setShowVariableDialog(false),
    [],
  );
  const handleAssetDoubleClick = (asset: Asset) => {
    const mapping = variableMappings[activeVariableIndex];
    if (Array.isArray(mapping?.defaultValue)) {
      const newArray = [...mapping.defaultValue];
      newArray[activeArrayItemIndex] = asset.url;
      handleDefaultValueChange(activeVariableIndex, newArray);
    } else {
      handleDefaultValueChange(activeVariableIndex, asset.url);
    }
    handleVariableDialogClose();
  };

  const handleCmsItemValue = (cmsItemAssetUrl: string) => {
    const mapping = variableMappings[activeVariableIndex];
    if (Array.isArray(mapping?.defaultValue)) {
      const newArray = [...mapping.defaultValue];
      newArray[activeArrayItemIndex] = cmsItemAssetUrl;
      handleDefaultValueChange(activeVariableIndex, newArray);
    } else {
      handleDefaultValueChange(activeVariableIndex, cmsItemAssetUrl);
    }
    handleDialogClose();
    handleVariableDialogClose();
  };

  const columns: AppColumnDef<TriggerVariableConfig>[] = useMemo(
    () => [
      {
        accessorKey: "name",
        header: t("Name"),
        cell: ({ row }) => {
          const mapping = row.original;
          return (
            <div className="flex flex-col gap-0.5">
              <span>
                {mapping.name}
                {mapping.required && (
                  <span className="text-destructive" aria-label={t("Required")}>
                    {" *"}
                  </span>
                )}
              </span>
              {mapping.stale && (
                <span className="text-xs text-warning">
                  {t("No longer in this deployment")}
                </span>
              )}
              {isMissing(mapping) && (
                <span className="text-xs text-destructive">
                  {t("Needs a value")}
                </span>
              )}
            </div>
          );
        },
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
              showVariableDialog={
                showVariableDialog && activeVariableIndex === row.index
              }
              onVariableDialogOpen={handleVariableDialogOpen}
              onVariableDialogClose={handleVariableDialogClose}
              onDefaultValueChange={handleDefaultValueChange}
              onAssetDialogOpen={handleDialogOpen}
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
              {row.original.stale && (
                <Tooltip>
                  <TooltipTrigger
                    render={
                      <IconButton
                        size="sm"
                        variant="ghost"
                        icon={<TrashIcon />}
                        onClick={() => handleRemove(row.index)}
                      />
                    }
                  />
                  <TooltipContent>{t("Remove value")}</TooltipContent>
                </Tooltip>
              )}
              {!row.original.stale &&
                row.original.deploymentDefault !== undefined && (
                  <Tooltip>
                    <TooltipTrigger
                      render={
                        <IconButton
                          size="sm"
                          variant="ghost"
                          icon={<ArrowUDownLeftIcon />}
                          onClick={() => handleResetToDefault(row.index)}
                          disabled={isAtDefault(row.original)}
                        />
                      }
                    />
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
    [
      activeVariableIndex,
      handleDefaultValueChange,
      handleRemove,
      handleResetToDefault,
      handleVariableDialogClose,
      handleVariableDialogOpen,
      isAtDefault,
      showVariableDialog,
      t,
    ],
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
          <DialogFooter className="flex items-center justify-end gap-2 p-4">
            {missingCount > 0 && (
              <p className="mr-auto text-xs text-destructive">
                {t("{{count}} required variables need a value", {
                  count: missingCount,
                })}
              </p>
            )}
            <Button variant="outline" onClick={handleCancel}>
              {t("Cancel")}
            </Button>
            <Button onClick={handleConfirm}>{t("Apply")}</Button>
          </DialogFooter>
        </div>
      </DialogContent>

      {showDialog === "assets" && (
        <AssetsDialog
          onDialogClose={handleDialogClose}
          onAssetSelect={handleAssetDoubleClick}
        />
      )}
      {showDialog === "cms" && (
        <CmsIntegrationDialog
          onDialogClose={handleDialogClose}
          onCmsItemValue={handleCmsItemValue}
        />
      )}
    </Dialog>
  );
};

export default TriggerWorkflowVariablesMappingDialog;
