import { Plus } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import { Button, FlowLogo, DataTable as Table } from "@flow/components";
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
  ];

  return selectedDeployment ? (
    <div className="flex flex-1">
      <DeploymentDetails
        selectedDeployment={selectedDeployment}
        onDeploymentUpdate={handleDeploymentUpdate}
        setDeploymentToBeDeleted={setDeploymentToBeDeleted}
      />
      <DeploymentDeletionDialog
        deploymentToBeDeleted={deploymentToBeDeleted}
        setDeploymentToBeDeleted={setDeploymentToBeDeleted}
        onDeleteDeployment={handleDeploymentDelete}
      />
    </div>
  ) : (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-6">
        <div className="flex h-[53px] items-center justify-between gap-2 border-b pb-4">
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
  );
};

export { DeploymentManager };
