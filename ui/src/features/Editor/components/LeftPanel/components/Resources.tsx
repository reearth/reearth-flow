import { HardDriveIcon } from "@phosphor-icons/react";

const Resources: React.FC = () => {
  return (
    <div className="flex flex-col gap-2 px-1">
      <div className="flex items-center gap-2">
        <HardDriveIcon className="size-[15px]" weight="thin" />
        <p className="text-sm dark:font-extralight">resource</p>
      </div>
      <div className="flex items-center gap-2">
        <HardDriveIcon className="size-[15px]" weight="thin" />
        <p className="text-sm dark:font-extralight">resource</p>
      </div>
      <div className="flex items-center gap-2">
        <HardDriveIcon className="size-[15px]" weight="thin" />
        <p className="text-sm dark:font-extralight">resource</p>
      </div>
      <div className="flex items-center gap-2">
        <HardDriveIcon className="size-[15px]" weight="thin" />
        <p className="text-sm dark:font-extralight">resource</p>
      </div>
    </div>
  );
};

export { Resources };
