import { ChalkboardTeacherIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { memo, useMemo } from "react";

import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogHeader,
  DialogTitle,
  DataTable as Table,
} from "@flow/components";
import { useWorkflowVariables } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { AnyWorkflowVariable, Project } from "@flow/types";

type Props = {
  isOpen: boolean;
  project: Project;
  onOpenChange: (open: boolean) => void;
  onCancel: () => void;
};

const SharedCanvasWorkflowVariablesDialog: React.FC<Props> = ({
  project,
  isOpen,
  onOpenChange,
  onCancel,
}) => {
  const t = useT();

  const { useGetWorkflowVariables } = useWorkflowVariables();
  const { workflowVariables } = useGetWorkflowVariables(project.id);

  const handleCancel = () => {
    onCancel();
    onOpenChange(false);
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
      },
      {
        accessorKey: "public",
        header: t("Public"),
        cell: ({ getValue }) => (getValue() ? t("Yes") : t("No")),
      },
      {
        accessorKey: "required",
        header: t("Required"),
        cell: ({ getValue }) => (getValue() ? t("Yes") : t("No")),
      },
    ],
    [t],
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
                  {t("Workflow Variables")}
                </div>
              </div>
            </DialogTitle>
          </DialogHeader>
          <div className="flex h-full min-h-0">
            <DialogContentSection className="flex min-h-0 flex-3 flex-col">
              <DialogContentSection className="mt-4 min-h-0 flex-1 overflow-hidden">
                <Table
                  columns={columns}
                  data={workflowVariables}
                  showOrdering={false}
                />
              </DialogContentSection>
            </DialogContentSection>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default memo(SharedCanvasWorkflowVariablesDialog);
