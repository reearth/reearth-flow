import { NewRun, StatusContent, RunDetails } from "./components";
import useHooks from "./hooks";

type Status = "running" | "queued" | "completed";

const RunsManager: React.FC = () => {
  const { tab, statusLabels, selectedRun, runs, handleRunSelect } = useHooks();

  return (
    <div className="flex-1">
      {tab === "new" ? (
        <NewRun />
      ) : isList(tab) ? (
        <StatusContent
          label={statusLabels[tab as Status]}
          runs={runs}
          onRunSelect={handleRunSelect}
        />
      ) : (
        <RunDetails selectedRun={selectedRun} />
      )}
    </div>
  );
};

export { RunsManager };

function isList(value: string) {
  return !!(
    value === "running" ||
    value === "queued" ||
    value === "completed" ||
    value === "all"
  );
}
