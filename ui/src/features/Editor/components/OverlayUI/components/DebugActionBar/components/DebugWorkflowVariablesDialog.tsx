import { ChalkboardTeacherIcon } from "@phosphor-icons/react";
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
  VariableRow,
} from "@flow/components";
import AssetsDialog from "@flow/features/AssetsDialog";
import CmsIntegrationDialog from "@flow/features/CmsIntegrationDialog";
import { useT } from "@flow/lib/i18n";
import { AnyWorkflowVariable, Asset } from "@flow/types";

type Props = {
  debugRunWorkflowVariables?: AnyWorkflowVariable[];
  onDebugRunVariableValueChange: (index: number, newValue: any) => void;
  onDebugRunStart: () => Promise<void>;
  onDialogClose: () => void;
};
type DialogOptions = "assets" | "cms" | undefined;

const DebugWorkflowVariablesDialog: React.FC<Props> = ({
  debugRunWorkflowVariables,
  onDebugRunVariableValueChange,
  onDebugRunStart,
  onDialogClose,
}) => {
  const [startingDebugRun, setStartingDebugRun] = useState(false);
  const [showDialog, setShowDialog] = useState<DialogOptions>(undefined);
  const [activeVariableIndex, setActiveVariableIndex] = useState<number>(0);
  const [showVariableDialog, setShowVariableDialog] = useState(false);

  const handleAssetDialogOpen = (dialog: DialogOptions) => {
    setShowDialog(dialog);
  };
  const handleDialogClose = () => setShowDialog(undefined);
  const handleVariableDialogOpen = useCallback((index: number) => {
    setActiveVariableIndex(index);
    setShowVariableDialog(true);
  }, []);
  const handleVariableDialogClose = useCallback(() => setShowVariableDialog(false), []);
  const handleAssetDoubleClick = (asset: Asset) => {
    onDebugRunVariableValueChange?.(activeVariableIndex, asset.url);
    handleVariableDialogClose();
  };

  const handleCmsItemValue = (cmsItemAssetUrl: string) => {
    onDebugRunVariableValueChange?.(activeVariableIndex, cmsItemAssetUrl);
    handleDialogClose();
    handleVariableDialogClose();
  };

  const t = useT();
  const handleDebugRunStart = async () => {
    setStartingDebugRun(true);
    await onDebugRunStart();
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
              showVariableDialog={showVariableDialog && activeVariableIndex === row.index}
              onVariableDialogOpen={handleVariableDialogOpen}
              onVariableDialogClose={handleVariableDialogClose}
              onAssetDialogOpen={handleAssetDialogOpen}
              onDefaultValueChange={onDebugRunVariableValueChange}
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
    ],
    [activeVariableIndex, handleVariableDialogClose, handleVariableDialogOpen, onDebugRunVariableValueChange, showVariableDialog, t],
  );

  return (
    <Dialog open onOpenChange={onDialogClose}>
      <DialogContent
        className="h-[50vh]"
        size="2xl"
        position="off-center"
        onInteractOutside={(e) => e.preventDefault()}>
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
                  data={debugRunWorkflowVariables}
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
