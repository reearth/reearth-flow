import type { ReactNode } from "react";

type Props = {
  className?: string;
  textClassName?: string;
  text: string;
  icon?: ReactNode;
  children?: ReactNode;
};

const BasicBoiler: React.FC<Props> = ({
  className,
  textClassName,
  text,
  icon,
  children,
}) => {
  return (
    <div
      className={`flex w-full flex-1 items-center justify-center text-lg dark:text-xl ${className}`}>
      <div className="flex flex-col items-center gap-6">
        {icon}
        <p className={`font-light dark:font-thin ${textClassName}`}>{text}</p>
        {children}
      </div>
    </div>
  );
};

export default BasicBoiler;
