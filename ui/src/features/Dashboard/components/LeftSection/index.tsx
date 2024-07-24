import { RunsSection, WorkspaceSection } from "./components";

const LeftSection: React.FC = () => {
  return (
    <div className="flex w-[250px] flex-col justify-between gap-[8px] border-r bg-secondary">
      <div className="flex flex-col gap-8">
        <RunsSection />
        <WorkspaceSection />
      </div>
    </div>
  );
};

export { LeftSection };
