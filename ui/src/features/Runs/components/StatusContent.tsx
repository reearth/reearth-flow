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
  <div className="flex flex-1 flex-col gap-4 px-6 pb-2 pt-6">
    <div className="flex h-[36px] items-center">
      <p className="text-xl font-extralight">{label}</p>
    </div>
    <div className="w-full border-b" />
    <div className="ml-8 mt-4 flex max-w-[1200px] flex-col gap-6">
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
