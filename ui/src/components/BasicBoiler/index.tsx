import { ReactNode } from "react";

type Props = {
  className?: string;
  text: string;
  icon?: ReactNode;
};

const BasicBoiler: React.FC<Props> = ({ className, text, icon }) => {
  return (
    <div
      className={`flex w-full flex-1 items-center justify-center ${className}`}>
      <div className="flex flex-col items-center gap-6">
        {icon}
        <p className="text-xl font-thin">{text}</p>
      </div>
    </div>
  );
};

export default BasicBoiler;
