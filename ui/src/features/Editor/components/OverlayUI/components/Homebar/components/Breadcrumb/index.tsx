import { memo, useEffect, useState } from "react";

import { useCurrentProject } from "@flow/stores";

const Breadcrumb: React.FC = () => {
  // const [currentWorkspace] = useCurrentWorkspace();
  const [currentProject] = useCurrentProject();
  const [isHovered, setIsHovered] = useState<string[] | undefined>(undefined);

  // This clears selection so that the text doesn't stay selected when hovering over the breadcrumb again
  useEffect(() => {
    if (!isHovered && window.getSelection()) {
      window.getSelection()?.empty();
    }
  }, [isHovered]);

  return (
    <div
      className="flex flex-1 cursor-default items-center justify-center gap-1 select-none"
      onMouseLeave={() => setIsHovered(undefined)}>
      {/* <div className="flex items-center gap-2">
        <UsersThreeIcon weight="thin" size={18} />
        <p
          className={`max-w-[200px] truncate text-sm transition-all delay-0 duration-500 dark:font-light ${isHovered?.includes("workspace") ? "max-w-[50vw] delay-500 select-text" : undefined}`}
          onMouseEnter={() => setIsHovered((h) => [...(h ?? []), "workspace"])}>
          {currentWorkspace?.name}
        </p>
      </div>
      <CaretRightIcon /> */}
      <div className="flex items-center">
        <p
          className={`min-w-[100px] truncate text-center text-sm font-bold transition-all duration-500 ${
            isHovered?.includes("project")
              ? "max-w-[50vw] delay-500 select-text"
              : "max-w-[250px] delay-0"
          }`}
          onMouseEnter={() => setIsHovered((h) => [...(h ?? []), "project"])}>
          {currentProject?.name}
        </p>
        {/* <CaretRightIcon /> */}
      </div>
    </div>
  );
};

export default memo(Breadcrumb);
