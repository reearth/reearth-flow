import { ColumnDef } from "@tanstack/react-table";

import { DataTable as Table } from "@flow/components";
import { JOBS_FETCH_RATE } from "@flow/lib/gql/job/useQueries";
import { useT } from "@flow/lib/i18n";
import type { Job } from "@flow/types";
import { formatTimestamp } from "@flow/utils/timestamp";

import { JobRunDialog } from "./components";
import useHooks from "./hooks";

const JobsManager: React.FC = () => {
  const t = useT();
  const {
    // ref,
    jobs,
    openJobRunDialog,
    isFetching,
    isDebouncingSearch,
    currentPage,
    totalPages,
    sortOptions,
    currentSortValue,
    setSearchTerm,
    setOpenJobRunDialog,
    handleJobSelect,
    handleSortChange,
    setCurrentPage,
  } = useHooks();

  const columns: ColumnDef<Job>[] = [
    {
      accessorKey: "id",
      header: t("ID"),
    },
    {
      accessorKey: "deploymentDescription",
      header: t("Deployment Description"),
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

  return (
    <div className="flex h-full flex-1 flex-col">
      <div className="flex flex-1 flex-col gap-1 overflow-scroll pt-4 pr-3 pb-2 pl-2">
        <div className="flex h-[50px] shrink-0 items-center justify-between gap-2 border-b pb-4">
          <p className="text-lg font-light dark:font-extralight">{t("Jobs")}</p>
        </div>
        <Table
          columns={columns}
          showFiltering
          data={jobs}
          selectColumns
          enablePagination
          currentPage={currentPage}
          totalPages={totalPages}
          resultsPerPage={resultsPerPage}
          currentSortValue={currentSortValue}
          sortOptions={sortOptions}
          isFetching={isDebouncingSearch || isFetching}
          noResultsMessage={t("No Jobs")}
          onRowClick={handleJobSelect}
          onSortChange={handleSortChange}
          setSearchTerm={setSearchTerm}
          setCurrentPage={setCurrentPage}
        />
      </div>
      {openJobRunDialog && <JobRunDialog setShowDialog={setOpenJobRunDialog} />}
    </div>
  );
};

export { JobsManager };
