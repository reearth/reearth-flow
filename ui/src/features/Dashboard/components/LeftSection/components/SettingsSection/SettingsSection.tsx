import { DeploymentSettings, WorkspaceSettings } from "./components";

const SettingsSection: React.FC = () => {
  return (
    <div className="flex flex-1 items-end">
      <div className="flex flex-1 flex-col gap-4 p-4">
        <DeploymentSettings />
        <WorkspaceSettings />
      </div>
    </div>
  );
};

export { SettingsSection };
