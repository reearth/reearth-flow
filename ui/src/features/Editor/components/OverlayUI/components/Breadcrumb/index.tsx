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
    <div className="pointer-events-none absolute inset-x-0 top-0 flex shrink-0 justify-center [&>*]:pointer-events-auto">
      <div
        className="flex cursor-default select-none items-center gap-3 rounded-br-md px-2 py-1"
        onMouseLeave={() => setIsHovered(undefined)}>
        <p
          className={`max-w-[100px] truncate font-extralight transition-all delay-0 duration-500 ${isHovered?.includes("workspace") ? "max-w-[50vw] select-text delay-500" : undefined}`}
          onMouseEnter={() => setIsHovered(h => [...(h ?? []), "workspace"])}>
          {currentWorkspace?.name}
        </p>
        <p className="font-extralight">
          <CaretRight />
        </p>
        <p
          className={`max-w-[100px] truncate font-extralight  transition-all delay-0 duration-500 ${isHovered?.includes("project") ? "max-w-[50vw] select-text delay-500" : undefined}`}
          onMouseEnter={() => setIsHovered(h => [...(h ?? []), "project"])}>
          {currentProject?.name}
        </p>
      </div>
    </div>
  );
};

export { Breadcrumb };
