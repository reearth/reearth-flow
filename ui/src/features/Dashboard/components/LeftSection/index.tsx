import { MembersSection, RunsSection } from "./components";

const LeftSection: React.FC = () => {
  return (
    <div className="flex flex-col gap-[8px]">
      <RunsSection />
      <MembersSection />
    </div>
  );
};

export { LeftSection };
