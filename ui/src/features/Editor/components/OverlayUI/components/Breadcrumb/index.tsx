import { CaretRight } from "@phosphor-icons/react";
import { useEffect, useState } from "react";

import { useCurrentProject, useCurrentWorkspace } from "@flow/stores";

const Breadcrumb: React.FC = () => {
  const [currentWorkspace] = useCurrentWorkspace();
  const [currentProject] = useCurrentProject();
  const [isHovered, setIsHovered] = useState<string[] | undefined>(undefined);

  // This clears selection so that the text doesn't stay selected when hovering over the breadcrumb again
  useEffect(() => {
    if (!isHovered && window.getSelection()) {
      window.getSelection()?.empty();
    }
  }, [isHovered]);

  return (
    <div className="absolute top-0 left-0 right-0 flex flex-shrink-0 justify-center pointer-events-none [&>*]:pointer-events-auto">
      <div
        className="py-1 px-2 flex items-center gap-3 rounded-br-md cursor-default select-none"
        onMouseLeave={() => setIsHovered(undefined)}>
        <p
          className={`font-extralight text-zinc-400 max-w-[100px] truncate transition-all delay-0 duration-500 ${isHovered?.includes("workspace") ? "max-w-[50vw] delay-500 select-text" : undefined}`}
          onMouseEnter={() => setIsHovered(h => [...(h ?? []), "workspace"])}>
          {currentWorkspace?.name}
        </p>
        <p className="font-extralight text-zinc-500">
          <CaretRight />
        </p>
        <p
          className={`font-extralight text-zinc-400 max-w-[100px] truncate transition-all delay-0 duration-500 ${isHovered?.includes("project") ? "max-w-[50vw] delay-500 select-text" : undefined}`}
          onMouseEnter={() => setIsHovered(h => [...(h ?? []), "project"])}>
          {currentProject?.name}
        </p>
      </div>
    </div>
  );
};

export { Breadcrumb };
