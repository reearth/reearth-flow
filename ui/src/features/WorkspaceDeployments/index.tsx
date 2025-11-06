import {
  PencilLineIcon,
  PlayIcon,
  PlusIcon,
  TrashIcon,
} from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import {
  Button,
  ButtonWithTooltip,
  DataTable as Table,
} from "@flow/components";
import { DEPLOYMENT_FETCH_RATE } from "@flow/lib/gql/deployment/useQueries";
import { useT } from "@flow/lib/i18n";
import type { Deployment } from "@flow/types";
import { formatTimestamp } from "@flow/utils/timestamp";

import {
  DeploymentAddDialog,
  DeploymentDeletionDialog,
  DeploymentDetails,
  DeploymentEditDialog,
  DeploymentRunDialog,
} from "./components";
import useHooks from "./hooks";

const DeploymentManager: React.FC = () => {
  const t = useT();
  const {
    deployments,
    selectedDeployment,
    deploymentToBeDeleted,
    deploymentToBeRun,
    openDeploymentAddDialog,
    deploymentToBeEdited,
    isFetching,
    isDebouncingSearch,
    sortOptions,
    currentSortValue,
    currentPage,
    totalPages,
    setDeploymentToBeEdited,
    setOpenDeploymentAddDialog,
    setDeploymentToBeDeleted,
    setDeploymentToBeRun,
    setSearchTerm,
    handleDeploymentSelect,
    handleDeploymentDelete,
    handleDeploymentRun,
    handleDeploymentRunConfirmed,
    handleSortChange,
    setCurrentPage,
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
            <PlayIcon />
          </ButtonWithTooltip>
          <ButtonWithTooltip
            variant="outline"
            size="icon"
            tooltipText={t("Edit Deployment")}
            onClick={() => setDeploymentToBeEdited(row.row.original)}>
            <PencilLineIcon />
          </ButtonWithTooltip>
          <ButtonWithTooltip
            variant="destructive"
            size="icon"
            tooltipText={t("Delete Deployment")}
            onClick={() => setDeploymentToBeDeleted(row.row.original)}>
            <TrashIcon />
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
        <>
          <div className="flex flex-1 flex-col gap-1 pt-4 pr-3 pb-2 pl-2">
            <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
              <p className="text-lg dark:font-extralight">{t("Deployments")}</p>
              <Button
                className="flex gap-2"
                onClick={() => setOpenDeploymentAddDialog(true)}>
                <PlusIcon weight="thin" />
                <p className="text-xs dark:font-light">{t("New Deployment")}</p>
              </Button>
            </div>
            <div className="h-full flex-1 overflow-hidden">
              <Table
                columns={columns}
                data={deployments}
                selectColumns
                enablePagination
                currentPage={currentPage}
                totalPages={totalPages}
                resultsPerPage={resultsPerPage}
                currentSortValue={currentSortValue}
                sortOptions={sortOptions}
                showFiltering
                isFetching={isDebouncingSearch || isFetching}
                noResultsMessage={t("No Deployments")}
                onRowClick={handleDeploymentSelect}
                onSortChange={handleSortChange}
                setCurrentPage={setCurrentPage}
                setSearchTerm={setSearchTerm}
              />
            </div>
          </div>
          {openDeploymentAddDialog && (
            <DeploymentAddDialog setShowDialog={setOpenDeploymentAddDialog} />
          )}
        </>
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
      {deploymentToBeRun && (
        <DeploymentRunDialog
          deployment={deploymentToBeRun}
          onDeploymentRun={handleDeploymentRunConfirmed}
          onDialogClose={() => setDeploymentToBeRun(undefined)}
        />
      )}
    </>
  );
};

export { DeploymentManager };
