import { ColumnDef } from "@tanstack/react-table";

import {
  FlowLogo,
  LoadingSkeleton,
  DataTable as Table,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { JOBS_FETCH_RATE } from "@flow/lib/gql/job/useQueries";
import { useT } from "@flow/lib/i18n";
import type { Job } from "@flow/types";
import { formatTimestamp } from "@flow/utils/timestamp";

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
    isFetching,
    currentPage,
    setCurrentPage,
    totalPages,
    currentOrder,
    setCurrentOrder,
    handleCancelJob,
  } = useHooks();

  const columns: ColumnDef<Job>[] = [
    {
      accessorKey: "id",
      header: t("ID"),
    },
    {
      accessorKey: "deploymentId",
      header: t("Deployment Id"),
    },
    {
      accessorKey: "status",
      header: t("Status"),
    },
    {
      accessorKey: "startedAt",
      header: t("Started At"),
      cell: ({ getValue }) => formatTimestamp(getValue<string>()),
    },
    {
      accessorKey: "completedAt",
      header: t("Completed At"),
      cell: ({ getValue }) => formatTimestamp(getValue<string>()),
    },
  ];
  const resultsPerPage = JOBS_FETCH_RATE;

  return selectedJob ? (
    <div className="flex flex-1">
      <JobDetails selectedJob={selectedJob} onJobCancel={handleCancelJob} />
    </div>
  ) : (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-4 overflow-scroll px-6 pb-2 pt-4">
        <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
          <p className="text-lg dark:font-extralight">{t("Jobs")}</p>
        </div>
        {isFetching ? (
          <LoadingSkeleton />
        ) : jobs && jobs.length > 0 ? (
          <Table
            columns={columns}
            data={jobs}
            selectColumns
            showFiltering
            enablePagination
            onRowClick={handleJobSelect}
            currentPage={currentPage}
            setCurrentPage={setCurrentPage}
            totalPages={totalPages}
            resultsPerPage={resultsPerPage}
            currentOrder={currentOrder}
            setCurrentOrder={setCurrentOrder}
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
