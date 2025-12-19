import { ChalkboardTeacherIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import {
  DataTable as Table,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogHeader,
  DialogTitle,
  Button,
  DialogFooter,
} from "@flow/components";
import { TriggerVariableRow } from "@flow/features/WorkspaceTriggers/components/TriggerWorkflowVariables/components";
import { useT } from "@flow/lib/i18n";
import { AnyWorkflowVariable } from "@flow/types";

type Props = {
  debugRunStarted: boolean;
  debugRunWorkflowVariables?: AnyWorkflowVariable[];
  onDebugRunVariableValueChange: (index: number, newValue: any) => void;
  onDebugRunStart: () => Promise<void>;
  onDialogClose: () => void;
};

const DebugWorkflowVariablesDialog: React.FC<Props> = ({
  debugRunWorkflowVariables,
  debugRunStarted,
  onDebugRunVariableValueChange,
  onDebugRunStart,
  onDialogClose,
}) => {
  const t = useT();
  const handleDebugRunStart = async () => {
    await onDebugRunStart();
    onDialogClose();
  };
  const columns: ColumnDef<AnyWorkflowVariable>[] = [
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
          <TriggerVariableRow
            variable={row.original}
            index={row.index}
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
  ];

  return (
    <Dialog open>
      <DialogContent
        className="h-[50vh]"
        size="2xl"
        position="off-center"
        hideCloseButton
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
              disabled={debugRunStarted}
              onClick={onDialogClose}>
              {t("Cancel")}
            </Button>
            <Button onClick={handleDebugRunStart} disabled={debugRunStarted}>
              {debugRunStarted ? t("Starting...") : t("Start")}
            </Button>
          </DialogFooter>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default DebugWorkflowVariablesDialog;
