import type { ProjectVariable as ProjectVariableType } from "@flow/types";

type Props = {
  className?: string;
  projectVariable: ProjectVariableType;
};

const ProjectVariable: React.FC<Props> = ({ className, projectVariable }) => {
  return (
    <div className={`flex items-center rounded p-1 ${className}`}>
      <p className="flex-1 truncate text-sm">{projectVariable.name}</p>
      <p className="flex-1 truncate text-sm dark:font-extralight">
        {JSON.stringify(projectVariable.value)}
      </p>
    </div>
  );
};

export { ProjectVariable };
