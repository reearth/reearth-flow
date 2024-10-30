import { RunsSection, EndSection } from "./components";

type Props = {
  baseRoute?: "deployments" | "projects";
  workspaceId: string;
};

const LeftSection: React.FC<Props> = ({ baseRoute, workspaceId }) => {
  return (
    <div className="flex w-[250px] flex-col justify-between gap-[8px] border-r bg-secondary">
      <div className="flex flex-1 flex-col">
        <RunsSection />
        <EndSection baseRoute={baseRoute} workspaceId={workspaceId} />
      </div>
    </div>
  );
};

export { LeftSection };
