import { Run } from "@flow/types";

import { RunsTable } from "./RunsTable";

type Props = {
  label: string;
  runs: Run[];
  selectedRun?: Run;
  onRunSelect: (run: Run) => void;
};

const StatusContent: React.FC<Props> = ({
  label,
  runs,
  selectedRun,
  onRunSelect,
}) => (
  <div className="flex-1 p-8">
    <div className="flex items-center gap-2 text-lg font-extralight">
      <p className="">{label}</p>
    </div>
    <div className="mt-4 flex max-w-[1200px] flex-col gap-6">
      <div className="min-h-[50vh] overflow-auto rounded-md px-2">
        <RunsTable
          runs={runs}
          selectedRun={selectedRun}
          onRunSelect={onRunSelect}
        />
      </div>
    </div>
  </div>
);

export { StatusContent };
