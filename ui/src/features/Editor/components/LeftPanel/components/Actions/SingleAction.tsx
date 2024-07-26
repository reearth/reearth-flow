import { Action } from "@flow/types";

const ActionComponent: React.FC<Action> = ({ name, type, description, categories }) => {
  return (
    <div className="group my-2 cursor-pointer rounded p-2  hover:bg-accent hover:text-accent-foreground">
      <div className="flex w-full justify-between gap-1 py-2">
        <div className="w-3/5 break-words text-sm">{name}</div>
        <div className="h-5 rounded bg-popover px-[2px] text-xs text-popover-foreground ">
          {type}
        </div>
      </div>
      <div className="hidden group-hover:block">
        <div className="mb-2 text-xs leading-[0.85rem]">{description}</div>
        <div className="flex gap-1 text-xs text-primary *:rounded *:bg-primary-foreground *:p-[2px] *:lowercase">
          {categories.map(c => (
            <div key={c}>{c}</div>
          ))}
        </div>
      </div>
    </div>
  );
};

export { ActionComponent };
