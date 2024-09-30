import { DoubleArrowRightIcon } from "@radix-ui/react-icons";

import "./styles.css";

type Props = {
  className?: string;
  show?: boolean;
};

const Loading: React.FC<Props> = ({ className, show }) => {
  return (
    show && (
      <div className={`absolute left-0 top-0 z-40 flex h-screen w-full justify-center bg-secondary/85 ${className}`}>
        <div className="flex h-full items-center">
          <div className="flex flex-col items-center gap-1">
            <div className="flex gap-3">
              <DoubleArrowRightIcon className="size-[110px]" />
            </div>
            <p className="text-xl font-light">実行中...</p>
          </div>
        </div>
      </div>
    )
  );
};

export { Loading };
