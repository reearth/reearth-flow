import type { ProjectVar } from "@flow/types";

type Props = {
  className?: string;
  variable: ProjectVar;
};

const ProjectVariable: React.FC<Props> = ({ className, variable }) => {
  return (
    <div className={`flex items-center rounded p-1 ${className}`}>
      <p className="flex-1 truncate text-sm">{variable.key}</p>
      <p className="flex-1 truncate text-sm dark:font-extralight">
        {JSON.stringify(variable.value)}
      </p>
    </div>
  );
};

export { ProjectVariable };
