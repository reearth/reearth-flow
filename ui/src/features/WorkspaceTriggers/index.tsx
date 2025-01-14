import { ColumnDef } from "@tanstack/react-table";

import { FlowLogo, DataTable as Table } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import { Trigger } from "@flow/types/trigger";

import { TriggerDetails } from "./components";
import useHooks from "./hooks";

const TriggerManager: React.FC = () => {
  const t = useT();
  const {
    // ref,
    deployments,
    selectedDeployment,
    deploymentToBeDeleted,
    setDeploymentToBeDeleted,
    handleDeploymentSelect,
    handleDeploymentUpdate,
    handleDeploymentDelete,
  } = useHooks();

  const columns: ColumnDef<Trigger>[] = [
    {
      accessorKey: "id",
      header: t("Id"),
    },
    {
      accessorKey: "id",
      header: t("Deployment"),
    },
    {
      accessorKey: "eventSource",
      header: t("Event Source"),
    },
    {
      accessorKey: "createdAt",
      header: t("Time Interval"),
    },
    {
      accessorKey: "updatedAt",
      header: t("Updated At"),
    },
    {
      accessorKey: "lastTriggered",
      header: t("Last Triggered"),
    },
  ];

  return selectedDeployment ? (
    <div className="flex flex-1">
      <TriggerDetails
        selectedDeployment={selectedDeployment}
        onDeploymentUpdate={handleDeploymentUpdate}
        setDeploymentToBeDeleted={setDeploymentToBeDeleted}
      />
      {/* <TriggerDeletionDialog
        deploymentToBeDeleted={deploymentToBeDeleted}
        setDeploymentToBeDeleted={setDeploymentToBeDeleted}
        onDeleteDeployment={handleDeploymentDelete}
      /> */}
    </div>
  ) : (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-6">
        <div className="flex h-[53px] items-center justify-between gap-2 border-b pb-4">
          <p className="text-lg dark:font-extralight">{t("Triggers")}</p>
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
            text={t("No Triggers")}
            icon={<FlowLogo className="size-16 text-accent" />}
          />
        )}
      </div>
    </div>
  );
};

export { TriggerManager };
