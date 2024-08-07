import { Action } from "@flow/types";

type Props = {
  action: Action;
  selected: boolean | undefined;
  onSingleClick?: (name?: string) => void;
  onDoubleClick?: (name?: string) => void;
} & React.HTMLAttributes<HTMLDivElement>;

const ActionItem: React.FC<Props> = ({
  action,
  selected,
  onSingleClick,
  onDoubleClick,
  ...divProps
}) => {
  return (
    <div
      key={action.name}
      className={`group cursor-pointer rounded px-2 ${selected ? "bg-primary text-accent-foreground" : "hover:bg-primary hover:text-accent-foreground"}`}
      onClick={() => onSingleClick?.(action.name)}
      onDoubleClick={() => onDoubleClick?.(action.name)}
      {...divProps}
    >
      <div className="flex w-full justify-between gap-1 py-2">
        <div className="w-3/5 self-center break-words text-sm">
          <p className="self-center text-zinc-200">{action.name}</p>
        </div>
        <div
          className={`self-center rounded border bg-popover p-1 align-middle`}
        >
          <p className="self-center text-xs capitalize text-zinc-200">
            {action.type}
          </p>
        </div>
      </div>
      <div className="group-hover:block">
        <div className="mb-2 text-xs leading-[0.85rem]">
          {action.description}
        </div>
        <div className="flex flex-wrap gap-1 text-xs ">
          {action.categories.map((c) => (
            <div className="rounded border bg-popover p-[2px]" key={c}>
              <p className="text-zinc-400">{c}</p>
            </div>
          ))}
        </div>
      </div>
      <div className="border-b pb-2" />
    </div>
  );
};

export default ActionItem;
