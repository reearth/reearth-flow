import { Job } from "@flow/types";

import { JobsTable } from "./JobsTable";

type Props = {
  label: string;
  jobs: Job[];
  onJobSelect: (jobId: string) => void;
};

const StatusContent: React.FC<Props> = ({ label, jobs, onJobSelect }) => (
  <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
    <div className="flex h-[36px] items-center">
      <p className="text-xl dark:font-extralight">{label}</p>
    </div>
    <div className="w-full border-b" />
    <div className="mt-4 flex flex-col gap-6">
      <div className="min-h-[50vh] overflow-auto rounded-md">
        <JobsTable jobs={jobs} onJobSelect={onJobSelect} />
      </div>
    </div>
  </div>
);

export { StatusContent };
