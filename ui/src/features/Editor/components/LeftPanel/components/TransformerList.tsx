import { Lightning } from "@phosphor-icons/react";

const TransformerList: React.FC = () => {
  return (
    <div className="flex flex-col gap-2 px-1">
      <div className="flex items-center gap-2">
        <Lightning className="size-[15px]" weight="thin" />
        <p className="text-sm font-extralight">Transformer</p>
      </div>
      <div className="flex items-center gap-2">
        <Lightning className="size-[15px]" weight="thin" />
        <p className="text-sm font-extralight">Transformer</p>
      </div>
      <div className="flex items-center gap-2">
        <Lightning className="size-[15px]" weight="thin" />
        <p className="text-sm font-extralight">Transformer</p>
      </div>
      <div className="flex items-center gap-2">
        <Lightning className="size-[15px]" weight="thin" />
        <p className="text-sm font-extralight">Transformer</p>
      </div>
    </div>
  );
};

export { TransformerList };
