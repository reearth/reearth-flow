import { memo } from "react";

import { Transformer } from "@flow/types";

type Props = Transformer & {
  selected: boolean;
  onSelect: () => void;
};

const TransformerComponent: React.FC<Props> = ({
  name,
  type,
  description,
  categories,
  selected,
  onSelect,
}) => {
  return (
    <div
      className={`group cursor-pointer rounded px-2 ${selected ? "bg-primary text-accent-foreground" : "hover:bg-primary hover:text-accent-foreground"}`}
      onClick={onSelect}
    >
      <div className="flex w-full justify-between gap-1 py-2">
        <div className="w-3/5 self-center break-words text-sm">
          <p className="self-center text-zinc-200">{name}</p>
        </div>
        <div
          className={`self-center rounded border bg-popover p-1 align-middle`}
        >
          <p className="self-center text-xs text-zinc-200">{type}</p>
        </div>
      </div>
      <div className="group-hover:block">
        <div className="mb-2 text-xs leading-[0.85rem]">{description}</div>
        <div className="flex flex-wrap gap-1 text-xs ">
          {categories.map((c) => (
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

export default memo(TransformerComponent);
