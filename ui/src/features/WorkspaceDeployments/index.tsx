import { ColumnDef } from "@tanstack/react-table";

import { FlowLogo, DataTable as Table } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { Deployment } from "@flow/types";

import useHooks from "./hooks";

const DeploymentManager: React.FC = () => {
  const t = useT();
  const { deployments } = useHooks();

  const columns: ColumnDef<Deployment>[] = [
    {
      accessorKey: "description",
      header: t("Description"),
    },
    {
      accessorKey: "projectName",
      header: t("Project Name"),
    },
    {
      accessorKey: "version",
      header: t("Version"),
    },
    {
      accessorKey: "createdAt",
      header: t("Created At"),
    },
    {
      accessorKey: "updatedAt",
      header: t("Updated At"),
    },
    {
      accessorKey: "actions",
      header: t("Actions"),
    },
  ];

  return (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-6">
        <div className="flex h-[53px] items-center justify-between gap-2 border-b pb-4">
          <p className="text-lg dark:font-extralight">{t("Deployments")}</p>
          {/* <Button
            className="flex gap-2"
            variant="outline"
            // onClick={() => setOpenProjectAddDialog(true)}>
          >
            <Plus weight="thin" />
            <p className="text-xs dark:font-light">{t("New Deployment")}</p>
          </Button> */}
        </div>
        <div className="flex flex-1">
          {deployments && deployments.length > 0 ? (
            <Table
              columns={columns}
              data={deployments}
              selectColumns
              showFiltering
              enablePagination
              rowHeight={14}
            />
          ) : (
            <div className="flex w-full items-center justify-center">
              <div className="flex flex-col items-center gap-6">
                <FlowLogo className="size-16 text-accent" />
                <p className="text-xl font-thin">{t("No deployments.")}</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export { DeploymentManager };
