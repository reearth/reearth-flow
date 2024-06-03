import { RunsSection, WorkspaceSection } from "./components";

const LeftSection: React.FC = () => {
  return (
    <div className="flex flex-col justify-between bg-zinc-900/50 border-r border-zinc-700 w-[250px] gap-[8px]">
      <div className="flex flex-col gap-8">
        <RunsSection />
        <WorkspaceSection />
      </div>
    </div>
  );
};

export { LeftSection };
