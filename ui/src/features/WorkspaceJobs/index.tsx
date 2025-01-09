import { NewJob, StatusContent, JobDetails } from "./components";
import useHooks from "./hooks";

type Status = "running" | "queued" | "completed";

const JobsManager: React.FC = () => {
  const { tab, statusLabels, selectedJob, jobs, handleJobSelect } = useHooks();

  return (
    <div className="flex-1">
      {tab === "new" ? (
        <NewJob />
      ) : isList(tab) ? (
        <StatusContent
          label={statusLabels[tab as Status]}
          jobs={jobs}
          onJobSelect={handleJobSelect}
        />
      ) : (
        <JobDetails selectedJob={selectedJob} />
      )}
    </div>
  );
};

export { JobsManager };

function isList(value: string) {
  return !!(
    value === "running" ||
    value === "queued" ||
    value === "completed" ||
    value === "all"
  );
}
