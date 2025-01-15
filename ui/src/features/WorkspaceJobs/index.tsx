import { ColumnDef } from "@tanstack/react-table";

import { FlowLogo, DataTable as Table } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useT } from "@flow/lib/i18n";
import type { Job } from "@flow/types";

import { JobRunDialog, JobDetails } from "./components";
import useHooks from "./hooks";

const JobsManager: React.FC = () => {
  const t = useT();
  const {
    // ref,
    jobs,
    selectedJob,
    openJobRunDialog,
    setOpenJobRunDialog,
    handleJobSelect,
  } = useHooks();

  const columns: ColumnDef<Job>[] = [
    {
      accessorKey: "id",
      header: t("ID"),
    },
    {
      accessorKey: "projectName",
      header: t("Project Name"),
    },
    {
      accessorKey: "status",
      header: t("Status"),
    },
    {
      accessorKey: "startedAt",
      header: t("Started At"),
    },
    {
      accessorKey: "completedAt",
      header: t("Completed At"),
    },
  ];

  return selectedJob ? (
    <div className="flex flex-1">
      <JobDetails selectedJob={selectedJob} />
    </div>
  ) : (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-4">
        <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
          <p className="text-lg dark:font-extralight">{t("Jobs")}</p>
        </div>
        {jobs && jobs.length > 0 ? (
          <Table
            columns={columns}
            data={jobs}
            selectColumns
            showFiltering
            enablePagination
            rowHeight={14}
            onRowClick={handleJobSelect}
          />
        ) : (
          <BasicBoiler
            text={t("No Jobs")}
            icon={<FlowLogo className="size-16 text-accent" />}
          />
        )}
      </div>
      {openJobRunDialog && <JobRunDialog setShowDialog={setOpenJobRunDialog} />}
    </div>
  );
};

export { JobsManager };
