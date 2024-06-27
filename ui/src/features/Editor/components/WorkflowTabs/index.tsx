import { Plus, X } from "@phosphor-icons/react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  editingCustomTransformers?: { id: string; name: string }[];
};

const WorkflowTabs: React.FC<Props> = ({ editingCustomTransformers }) => {
  const t = useT();

  return (
    <div className="bg-zinc-800 w-[75vw]">
      <div className="flex flex-1 items-center bg-zinc-900/50 h-[29px]">
        <div className="flex justify-center items-center w-28 mx-1 px-[6px] py-[2px] rounded bg-zinc-700 cursor-pointer">
          <p className="text-xs text-center font-extralight text-zinc-300 truncate">
            {t("Main Workflow")}
          </p>
        </div>
        <div className="flex items-center gap-1 h-[29px] overflow-auto">
          {editingCustomTransformers?.length &&
            editingCustomTransformers.map(transformer => (
              <div
                key={transformer.id}
                className="flex justify-center items-center relative w-28 px-[6px] py-[2px] rounded text-zinc-400 transition-colors hover:bg-zinc-600 hover:text-zinc-300 cursor-pointer group">
                <X className="absolute right-[2px] w-[15px] h-[15px] hidden group-hover:bg-zinc-600 group-hover:block" />
                <p className="text-xs text-center font-extralight truncate group-hover:text-zinc-300">
                  {transformer.name}
                </p>
              </div>
            ))}
        </div>
        <div className="flex items-center">
          <IconButton
            className="h-[25px]"
            icon={<Plus weight="light" />}
            tooltipText={t("Create new sub workflow")}
          />
        </div>
      </div>
    </div>
  );
};

export { WorkflowTabs };
