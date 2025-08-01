import { useT } from "@flow/lib/i18n";

import useHooks from "./hooks";

const JobStatus: React.FC = () => {
  const t = useT();
  const { jobStatus } = useHooks();
  return jobStatus ? (
    <div className="flex items-center gap-2 rounded-md border border-primary bg-secondary/70 p-4 shadow-md shadow-secondary backdrop-blur-xs">
      <p className="text-xs font-light">{t("Debug Status: ")}</p>
      <p className="text-xs font-thin">{jobStatus ?? t("idle")}</p>
      <div
        className={`${
          jobStatus === "completed"
            ? "bg-success"
            : jobStatus === "running"
              ? "active-node-status"
              : jobStatus === "cancelled"
                ? "bg-warning"
                : jobStatus === "failed"
                  ? "bg-destructive"
                  : jobStatus === "queued"
                    ? "queued-node-status"
                    : "bg-primary"
        } size-3 rounded-full`}
      />
    </div>
  ) : null;
};

export default JobStatus;
