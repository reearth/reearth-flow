import type { UserParameter } from "@flow/types";

type Props = {
  className?: string;
  parameter: UserParameter;
};

const UserParameter: React.FC<Props> = ({ className, parameter }) => {
  return (
    <div className={`flex items-center rounded p-1 ${className}`}>
      <p className="flex-1 truncate text-sm">{parameter.name}</p>
      <p className="flex-1 truncate text-sm dark:font-extralight">
        {JSON.stringify(parameter.value)}
      </p>
    </div>
  );
};

export { UserParameter };
