import { HardDrive } from "@phosphor-icons/react";

const Resources: React.FC = () => {
  return (
    <div className="flex flex-col gap-2 px-1">
      <div className="flex gap-2 items-center">
        <HardDrive className="w-[15px] h-[15px]" weight="thin" />
        <p className="text-sm font-extralight">resource</p>
      </div>
      <div className="flex gap-2 items-center">
        <HardDrive className="w-[15px] h-[15px]" weight="thin" />
        <p className="text-sm font-extralight">resource</p>
      </div>
      <div className="flex gap-2 items-center">
        <HardDrive className="w-[15px] h-[15px]" weight="thin" />
        <p className="text-sm font-extralight">resource</p>
      </div>
      <div className="flex gap-2 items-center">
        <HardDrive className="w-[15px] h-[15px]" weight="thin" />
        <p className="text-sm font-extralight">resource</p>
      </div>
    </div>
  );
};

export { Resources };
