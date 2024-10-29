import { RunsSection, SettingsSection } from "./components";

const LeftSection: React.FC = () => {
  return (
    <div className="flex w-[250px] flex-col justify-between gap-[8px] border-r bg-secondary">
      <div className="flex flex-1 flex-col">
        <RunsSection />
        <SettingsSection />
      </div>
    </div>
  );
};

export { LeftSection };
