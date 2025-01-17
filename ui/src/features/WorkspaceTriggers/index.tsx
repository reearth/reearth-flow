import { PencilLine, Plus, Trash } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";

import {
  Button,
  ButtonWithTooltip,
  FlowLogo,
  DataTable as Table,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import { Trigger } from "@flow/types/trigger";

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
  } = useHooks();

  const columns: ColumnDef<Trigger>[] = [
    {
      accessorKey: "id",
      header: t("ID"),
    },
    {
      accessorKey: "eventSource",
      header: t("Event Source"),
    },
    {
      accessorKey: "lastTriggered",
      header: t("Last Triggered"),
    },
    {
      accessorKey: "quickActions",
      header: t("Quick Actions"),
      cell: (row) => (
        <div className="flex gap-2">
          <ButtonWithTooltip
            variant="outline"
            size="icon"
            tooltipText={t("Edit Trigger")}
            onClick={() => setTriggerToBeEdited(row.row.original)}>
            <PencilLine />
          </ButtonWithTooltip>
          <ButtonWithTooltip
            variant="destructive"
            size="icon"
            tooltipText={t("Delete Trigger")}
            onClick={() => setTriggerToBeDeleted(row.row.original)}>
            <Trash />
          </ButtonWithTooltip>
        </div>
      ),
    },
  ];

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
          <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-4">
            <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
              <p className="text-lg dark:font-extralight">{t("Triggers")}</p>
              <Button
                className="flex gap-2"
                variant="outline"
                onClick={() => setOpenTriggerAddDialog(true)}>
                <Plus weight="thin" />
                <p className="text-xs dark:font-light">{t("New Trigger")}</p>
              </Button>
            </div>
            {triggers && triggers.length > 0 ? (
              <Table
                columns={columns}
                data={triggers}
                selectColumns
                showFiltering
                enablePagination
                rowHeight={14}
                onRowClick={handleTriggerSelect}
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
