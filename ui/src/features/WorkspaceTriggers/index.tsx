import { PencilLineIcon, PlusIcon, TrashIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import {
  Button,
  ButtonWithTooltip,
  FlowLogo,
  LoadingTableSkeleton,
  DataTable as Table,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { TRIGGERS_FETCH_RATE } from "@flow/lib/gql/trigger/useQueries";
import { useT } from "@flow/lib/i18n";
import { Trigger } from "@flow/types";
import { formatTimestamp } from "@flow/utils";

import {
  TriggerAddDialog,
  TriggerDeletionDialog,
  TriggerDetails,
  TriggerEditDialog,
} from "./components";
import useHooks from "./hooks";

const TriggerManager: React.FC = () => {
  const t = useT();
  const {
    triggers,
    selectedTrigger,
    triggerToBeDeleted,
    openTriggerAddDialog,
    triggerToBeEdited,
    setTriggerToBeEdited,
    setOpenTriggerAddDialog,
    setTriggerToBeDeleted,
    handleTriggerSelect,
    handleTriggerDelete,
    currentPage,
    setCurrentPage,
    totalPages,
    currentOrder,
    setCurrentOrder,
    isFetching,
  } = useHooks();
  const columns: ColumnDef<Trigger>[] = [
    {
      accessorKey: "description",
      header: t("Trigger Description"),
    },
    {
      accessorKey: "deployment.description",
      header: t("Deployment Description"),
    },
    {
      accessorKey: "eventSource",
      header: t("Event Source"),
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
            variant="outline"
            size="icon"
            tooltipText={t("Update Trigger")}
            onClick={() => setTriggerToBeEdited(row.row.original)}>
            <PencilLineIcon />
          </ButtonWithTooltip>
          <ButtonWithTooltip
            variant="destructive"
            size="icon"
            tooltipText={t("Delete Trigger")}
            onClick={() => setTriggerToBeDeleted(row.row.original)}>
            <TrashIcon />
          </ButtonWithTooltip>
        </div>
      ),
    },
  ];
  const resultsPerPage = TRIGGERS_FETCH_RATE;
  return (
    <>
      {selectedTrigger ? (
        <div className="flex flex-1">
          <TriggerDetails
            selectedTrigger={selectedTrigger}
            setTriggerToBeDeleted={setTriggerToBeDeleted}
          />
        </div>
      ) : (
        <div className="flex h-full flex-1 flex-col">
          <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pt-4 pb-2">
            <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
              <p className="text-lg dark:font-extralight">{t("Triggers")}</p>
              <Button
                className="flex gap-2"
                onClick={() => setOpenTriggerAddDialog(true)}>
                <PlusIcon weight="thin" />
                <p className="text-xs dark:font-light">{t("New Trigger")}</p>
              </Button>
            </div>

            {isFetching ? (
              <LoadingTableSkeleton
                columns={columns.length}
                rows={10}
                hasQuickActions
                hasColumns
                hasOrdering
              />
            ) : triggers && triggers.length > 0 ? (
              <Table
                columns={columns}
                data={triggers}
                selectColumns
                enablePagination
                onRowClick={handleTriggerSelect}
                currentPage={currentPage}
                setCurrentPage={setCurrentPage}
                totalPages={totalPages}
                resultsPerPage={resultsPerPage}
                currentOrder={currentOrder}
                setCurrentOrder={setCurrentOrder}
              />
            ) : (
              <BasicBoiler
                text={t("No Triggers")}
                icon={<FlowLogo className="size-16 text-accent" />}
              />
            )}
          </div>
          {openTriggerAddDialog && (
            <TriggerAddDialog setShowDialog={setOpenTriggerAddDialog} />
          )}
        </div>
      )}
      {triggerToBeEdited && (
        <TriggerEditDialog
          selectedTrigger={triggerToBeEdited}
          onDialogClose={() => setTriggerToBeEdited(undefined)}
        />
      )}
      {triggerToBeDeleted && (
        <TriggerDeletionDialog
          triggerTobeDeleted={triggerToBeDeleted}
          setTriggerToBeDeleted={setTriggerToBeDeleted}
          onTriggerDelete={handleTriggerDelete}
        />
      )}
    </>
  );
};

export { TriggerManager };
