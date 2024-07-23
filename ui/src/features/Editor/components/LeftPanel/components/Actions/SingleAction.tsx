import { Action } from "@flow/types";

const ActionComponent: React.FC<Action> = ({ name, type, description, categories }) => {
  return (
    <div className="group my-2 cursor-pointer rounded bg-zinc-800 p-2 text-zinc-400 hover:bg-zinc-700 hover:text-zinc-300">
      <div className="flex w-full justify-between gap-1 py-2">
        <div className="w-3/5 break-words text-sm">{name}</div>
        <div className="h-5 rounded bg-stone-400 px-[2px] text-xs text-zinc-900">{type}</div>
      </div>
      <div className="hidden group-hover:block">
        <div className="mb-2 text-xs leading-[0.85rem]">{description}</div>
        <div className="flex gap-1 text-xs text-zinc-900 *:rounded *:bg-stone-400 *:p-[2px] *:lowercase">
          {categories.map(c => (
            <div key={c}>{c}</div>
          ))}
        </div>
      </div>
    </div>
  );
};

export { ActionComponent };
