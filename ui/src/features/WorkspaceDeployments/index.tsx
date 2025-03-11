import { PencilLine, Play, Plus, Trash } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import {
  Button,
  ButtonWithTooltip,
  FlowLogo,
  LoadingSkeleton,
  DataTable as Table,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { DEPLOYMENT_FETCH_RATE } from "@flow/lib/gql/deployment/useQueries";
import { useT } from "@flow/lib/i18n";
import type { Deployment } from "@flow/types";
import { formatTimestamp } from "@flow/utils/timestamp";

import {
  DeploymentAddDialog,
  DeploymentDeletionDialog,
  DeploymentDetails,
  DeploymentEditDialog,
} from "./components";
import useHooks from "./hooks";

const DeploymentManager: React.FC = () => {
  const t = useT();
  const {
    deployments,
    selectedDeployment,
    deploymentToBeDeleted,
    openDeploymentAddDialog,
    deploymentToBeEdited,
    isFetching,
    currentPage,
    totalPages,
    currentOrder,
    setDeploymentToBeEdited,
    setOpenDeploymentAddDialog,
    setDeploymentToBeDeleted,
    handleDeploymentSelect,
    handleDeploymentDelete,
    handleDeploymentRun,
    setCurrentPage,
    setCurrentOrder,
  } = useHooks();
  const resultsPerPage = DEPLOYMENT_FETCH_RATE;
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
      accessorKey: "updatedAt",
      header: t("Updated At"),
      cell: ({ getValue }) => formatTimestamp(getValue<string>()),
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
            variant="outline"
            size="icon"
            tooltipText={t("Edit Deployment")}
            onClick={() => setDeploymentToBeEdited(row.row.original)}>
            <PencilLine />
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
      {selectedDeployment ? (
        <div className="flex flex-1">
          <DeploymentDetails
            selectedDeployment={selectedDeployment}
            setDeploymentToBeDeleted={setDeploymentToBeDeleted}
            onDeploymentRun={handleDeploymentRun}
          />
        </div>
      ) : (
        <div className="flex h-full flex-1 flex-col">
          <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-4">
            <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
              <p className="text-lg dark:font-extralight">{t("Deployments")}</p>
              <Button
                className="flex gap-2"
                onClick={() => setOpenDeploymentAddDialog(true)}>
                <Plus weight="thin" />
                <p className="text-xs dark:font-light">{t("New Deployment")}</p>
              </Button>
            </div>
            {isFetching ? (
              <LoadingSkeleton />
            ) : deployments && deployments.length > 0 ? (
              <Table
                columns={columns}
                data={deployments}
                selectColumns
                showFiltering
                enablePagination
                onRowClick={handleDeploymentSelect}
                currentPage={currentPage}
                setCurrentPage={setCurrentPage}
                totalPages={totalPages}
                resultsPerPage={resultsPerPage}
                currentOrder={currentOrder}
                setCurrentOrder={setCurrentOrder}
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
      {deploymentToBeEdited && (
        <DeploymentEditDialog
          selectedDeployment={deploymentToBeEdited}
          onDialogClose={() => setDeploymentToBeEdited(undefined)}
        />
      )}
      {deploymentToBeDeleted && (
        <DeploymentDeletionDialog
          deploymentToBeDeleted={deploymentToBeDeleted}
          setDeploymentToBeDeleted={setDeploymentToBeDeleted}
          onDeploymentDelete={handleDeploymentDelete}
        />
      )}
    </>
  );
};

export { DeploymentManager };
