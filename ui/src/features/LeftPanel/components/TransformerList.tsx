import { Lightning } from "@phosphor-icons/react";

const TransformerList: React.FC = () => {
  return (
    <div className="flex flex-col gap-2 px-1">
      <div className="flex gap-2 items-center">
        <Lightning className="w-[15px] h-[15px]" weight="thin" />
        <p className="text-sm font-extralight">Transformer</p>
      </div>
      <div className="flex gap-2 items-center">
        <Lightning className="w-[15px] h-[15px]" weight="thin" />
        <p className="text-sm font-extralight">Transformer</p>
      </div>
      <div className="flex gap-2 items-center">
        <Lightning className="w-[15px] h-[15px]" weight="thin" />
        <p className="text-sm font-extralight">Transformer</p>
      </div>
      <div className="flex gap-2 items-center">
        <Lightning className="w-[15px] h-[15px]" weight="thin" />
        <p className="text-sm font-extralight">Transformer</p>
      </div>
    </div>
  );
};

export { TransformerList };
