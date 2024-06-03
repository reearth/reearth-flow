import { X } from "@phosphor-icons/react";

import { useT } from "@flow/lib/i18n";

type Props = {
  editingCustomTransformers?: { id: string; name: string }[];
};

const CanvasTabs: React.FC<Props> = ({ editingCustomTransformers }) => {
  const t = useT();

  return editingCustomTransformers?.length ? (
    <div className="absolute left-0 bottom-0 flex bg-zinc-900 border-t border-r border-zinc-700 rounded-tr">
      <div className="w-28 px-2 py-0.5 bg-zinc-800 border-r border-zinc-700 cursor-pointer">
        <p className="text-xs text-center text-zinc-400 truncate">{t("Main Canvas")}</p>
      </div>
      <div className="flex max-w-[50vw] overflow-auto">
        {editingCustomTransformers.map((transformer, idx) => (
          <div
            key={transformer.id}
            className={`relative w-28 px-2 py-0.5 border-r border-zinc-700 text-zinc-500 transition-colors hover:bg-zinc-800 hover:text-zinc-400 cursor-pointer group ${editingCustomTransformers.length === idx + 1 ? "rounded-tr" : undefined}`}>
            <X className="absolute right-[2px] w-[15px] h-[15px] hidden bg-zinc-800 group-hover:block" />
            <p className="text-xs text-center truncate">{transformer.name}</p>
          </div>
        ))}
      </div>
    </div>
  ) : null;
};

export { CanvasTabs };
