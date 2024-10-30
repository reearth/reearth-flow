import { RunsSection, EndSection } from "./components";

type Props = {
  baseRoute?: "deployments" | "projects";
};

const LeftSection: React.FC<Props> = ({ baseRoute }) => {
  return (
    <div className="flex w-[250px] flex-col justify-between gap-[8px] border-r bg-secondary">
      <div className="flex flex-1 flex-col">
        <RunsSection />
        <EndSection baseRoute={baseRoute} />
      </div>
    </div>
  );
};

export { LeftSection };
