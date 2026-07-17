import {
  ArrowUDownLeftIcon,
  ChalkboardTeacherIcon,
  PencilLineIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useMemo, useState } from "react";

import {
  DataTable as Table,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogHeader,
  DialogTitle,
  Button,
  DialogFooter,
  IconButton,
  VariableRow,
} from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import CmsIntegrationDialog from "@flow/features/CmsIntegrationDialog";
import { useT } from "@flow/lib/i18n";
import { AnyWorkflowVariable, Asset } from "@flow/types";

type Props = {
  debugRunWorkflowVariables?: AnyWorkflowVariable[];
  workflowVariableDefaults?: AnyWorkflowVariable[];
  onDebugRunStart: (variables?: AnyWorkflowVariable[]) => Promise<void>;
  onDialogClose: () => void;
};
type DialogOptions = "assets" | "cms" | undefined;

const DebugWorkflowVariablesDialog: React.FC<Props> = ({
  debugRunWorkflowVariables,
  workflowVariableDefaults,
  onDebugRunStart,
  onDialogClose,
}) => {
  const t = useT();

  // Local draft state for workflow variables, initialized with the debug run variables if provided
  const [variables, setVariables] = useState<AnyWorkflowVariable[]>(
    () => debugRunWorkflowVariables ?? [],
  );

  const [startingDebugRun, setStartingDebugRun] = useState(false);
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const [activeVariableIndex, setActiveVariableIndex] = useState<number>(0);
  const [activeArrayItemIndex, setActiveArrayItemIndex] = useState<number>(0);
  const [showVariableDialog, setShowVariableDialog] = useState(false);

  const handleDefaultValueChange = useCallback(
    (index: number, newValue: any) => {
      setVariables((prev) =>
        prev.map((variable, i) =>
          i === index ? { ...variable, defaultValue: newValue } : variable,
        ),
      );
    },
    [],
  );

  const handleResetToDefault = useCallback(
    (index: number, variableId: string) => {
      const original = workflowVariableDefaults?.find(
        (defaultVariable) => defaultVariable.id === variableId,
      );
      if (!original) return;
      setVariables((prev) =>
        prev.map((variable, i) =>
          i === index
            ? { ...variable, defaultValue: original.defaultValue }
            : variable,
        ),
      );
    },
    [workflowVariableDefaults],
  );

  const isAtDefault = useCallback(
    (variable: AnyWorkflowVariable): boolean => {
      const original = workflowVariableDefaults?.find(
        (defaultVariable) => defaultVariable.id === variable.id,
      );
      if (!original) return true;
      return (
        JSON.stringify(variable.defaultValue) ===
        JSON.stringify(original.defaultValue)
      );
    },
    [workflowVariableDefaults],
  );

  const handleAssetDialogOpen = (dialog: DialogOptions) => {
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
    const variable = variables[activeVariableIndex];
    if (Array.isArray(variable?.defaultValue)) {
      const newArray = [...variable.defaultValue];
      newArray[activeArrayItemIndex] = asset.url;
      handleDefaultValueChange(activeVariableIndex, newArray);
    } else {
      handleDefaultValueChange(activeVariableIndex, asset.url);
    }
    handleVariableDialogClose();
  };

  const handleCmsItemValue = (cmsItemAssetUrl: string) => {
    const variable = variables[activeVariableIndex];
    if (Array.isArray(variable?.defaultValue)) {
      const newArray = [...variable.defaultValue];
      newArray[activeArrayItemIndex] = cmsItemAssetUrl;
      handleDefaultValueChange(activeVariableIndex, newArray);
    } else {
      handleDefaultValueChange(activeVariableIndex, cmsItemAssetUrl);
    }
    handleDialogClose();
    handleVariableDialogClose();
  };

  const handleDebugRunStart = async () => {
    setStartingDebugRun(true);
    await onDebugRunStart(variables);
    setStartingDebugRun(false);
    onDialogClose();
  };

  const columns: ColumnDef<AnyWorkflowVariable>[] = useMemo(
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
              showVariableDialog={
                showVariableDialog && activeVariableIndex === row.index
              }
              onVariableDialogClose={handleVariableDialogClose}
              onAssetDialogOpen={handleAssetDialogOpen}
              onDefaultValueChange={handleDefaultValueChange}
            />
          );
        },
      },
      {
        accessorKey: "required",
        header: t("Required"),
        cell: ({ getValue }) => (getValue() ? t("Yes") : t("No")),
      },
      {
        accessorKey: "public",
        header: t("Public"),
        cell: ({ getValue }) => (getValue() ? t("Yes") : t("No")),
      },
      {
        id: "actions",
        header: t("Actions"),
        cell: ({ row }) => (
          <div>
            <IconButton
              size="sm"
              variant="ghost"
              icon={<PencilLineIcon />}
              tooltipText={t("Edit default value")}
              onClick={() => handleVariableDialogOpen(row.index)}
            />
            <IconButton
              size="sm"
              variant="ghost"
              icon={<ArrowUDownLeftIcon />}
              tooltipText={t("Reset to default")}
              onClick={() => handleResetToDefault(row.index, row.original.id)}
              disabled={isAtDefault(row.original)}
            />
          </div>
        ),
        size: 100,
      },
    ],
    [
      activeVariableIndex,
      handleDefaultValueChange,
      handleResetToDefault,
      handleVariableDialogClose,
      handleVariableDialogOpen,
      isAtDefault,
      showVariableDialog,
      t,
    ],
  );

  return (
    <Dialog open disablePointerDismissal onOpenChange={onDialogClose}>
      <DialogContent className="h-[50vh]" size="2xl" position="off-center">
        <div className="flex h-full flex-col">
          <DialogHeader>
            <DialogTitle>
              <div className="flex items-center justify-between gap-2">
                <div className="flex items-center gap-2">
                  <ChalkboardTeacherIcon />
                  {t("Workflow Variables")}
                </div>
              </div>
            </DialogTitle>
          </DialogHeader>
          <div className="flex h-full min-h-0">
            <DialogContentSection className="flex min-h-0 flex-3 flex-col">
              <DialogContentSection className="min-h-0 flex-1 overflow-hidden">
                <Table
                  columns={columns}
                  data={variables}
                  showOrdering={false}
                />
              </DialogContentSection>
            </DialogContentSection>
          </div>
          <DialogFooter className="flex justify-end gap-2 p-4">
            <Button
              variant="outline"
              disabled={startingDebugRun}
              onClick={onDialogClose}>
              {t("Cancel")}
            </Button>
            <Button onClick={handleDebugRunStart} disabled={startingDebugRun}>
              {startingDebugRun ? t("Starting...") : t("Start")}
            </Button>
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

export default DebugWorkflowVariablesDialog;
