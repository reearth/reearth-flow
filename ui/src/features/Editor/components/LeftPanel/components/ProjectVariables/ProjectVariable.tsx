export type VarType = "string" | "number" | "boolean" | "array" | "object";

export type ProjectVar = {
  key: string;
  value: any;
  type: VarType;
};

type Props = {
  className?: string;
  variable: ProjectVar;
};

const ProjectVariable: React.FC<Props> = ({ className, variable }) => {
  return (
    <tr>
      <td className={`rounded-l ${className}`}>{variable.key}</td>
      <td
        className={`text-wrap break-words rounded-r text-sm dark:font-extralight ${className}`}>
        {variable.value}
      </td>
    </tr>
  );
};

export { ProjectVariable };
