import { Play, Plus, Trash } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import {
  Button,
  ButtonWithTooltip,
  FlowLogo,
  DataTable as Table,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import type { Deployment } from "@flow/types";

import {
  DeploymentAddDialog,
  DeploymentDeletionDialog,
  DeploymentDetails,
} from "./components";
import useHooks from "./hooks";

const DeploymentManager: React.FC = () => {
  const t = useT();
  const {
    // ref,
    deployments,
    selectedDeployment,
    deploymentToBeDeleted,
    openDeploymentAddDialog,
    setOpenDeploymentAddDialog,
    setDeploymentToBeDeleted,
    handleDeploymentSelect,
    handleDeploymentUpdate,
    handleDeploymentDelete,
    handleDeploymentRun,
  } = useHooks();

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
      accessorKey: "quickActions",
      header: t("Quick Actions"),
      cell: (row) => (
        <div className="flex gap-2">
          <ButtonWithTooltip
            variant="default"
            size="icon"
            tooltipText={t("Run Deployment")}
            onClick={() => handleDeploymentRun(row.row.original)}>
            <Play />
          </ButtonWithTooltip>
          <ButtonWithTooltip
            variant="destructive"
            size="icon"
            tooltipText={t("Delete Deployment")}
            onClick={() => setDeploymentToBeDeleted(row.row.original)}>
            <Trash />
          </ButtonWithTooltip>
        </div>
      ),
    },
  ];

  return (
    <>
      {deploymentToBeDeleted && (
        <DeploymentDeletionDialog
          deploymentToBeDeleted={deploymentToBeDeleted}
          setDeploymentToBeDeleted={setDeploymentToBeDeleted}
          onDeploymentDelete={handleDeploymentDelete}
        />
      )}
      {selectedDeployment ? (
        <div className="flex flex-1">
          <DeploymentDetails
            selectedDeployment={selectedDeployment}
            setDeploymentToBeDeleted={setDeploymentToBeDeleted}
            onDeploymentRun={handleDeploymentRun}
            onDeploymentUpdate={handleDeploymentUpdate}
          />
        </div>
      ) : (
        <div className="flex h-full flex-1 flex-col">
          <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-4">
            <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
              <p className="text-lg dark:font-extralight">{t("Deployments")}</p>
              <Button
                className="flex gap-2"
                variant="outline"
                onClick={() => setOpenDeploymentAddDialog(true)}>
                <Plus weight="thin" />
                <p className="text-xs dark:font-light">{t("New Deployment")}</p>
              </Button>
            </div>
            {deployments && deployments.length > 0 ? (
              <Table
                columns={columns}
                data={deployments}
                selectColumns
                showFiltering
                enablePagination
                rowHeight={14}
                onRowClick={handleDeploymentSelect}
              />
            ) : (
              <BasicBoiler
                text={t("No Deployments")}
                icon={<FlowLogo className="size-16 text-accent" />}
              />
            )}
          </div>
          {openDeploymentAddDialog && (
            <DeploymentAddDialog setShowDialog={setOpenDeploymentAddDialog} />
          )}
        </div>
      )}
    </>
  );
};

export { DeploymentManager };
