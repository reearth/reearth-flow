import { SquaresFour, UsersThree } from "@phosphor-icons/react";
import { memo, useEffect, useState } from "react";

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
    <div
      className="flex cursor-default items-center gap-2 select-none"
      onMouseLeave={() => setIsHovered(undefined)}>
      <UsersThree weight="thin" size={18} />
      <p
        className={`max-w-[200px] truncate text-sm transition-all delay-0 duration-500 dark:font-thin ${isHovered?.includes("workspace") ? "max-w-[50vw] delay-500 select-text" : undefined}`}
        onMouseEnter={() => setIsHovered((h) => [...(h ?? []), "workspace"])}>
        {currentWorkspace?.name}
      </p>
      <p className="text-sm font-thin text-accent-foreground">{"/"}</p>
      <SquaresFour weight="thin" size={18} />
      <p
        className={`max-w-[200px] truncate text-sm transition-all delay-0 duration-500 dark:font-thin ${isHovered?.includes("project") ? "max-w-[50vw] delay-500 select-text" : undefined}`}
        onMouseEnter={() => setIsHovered((h) => [...(h ?? []), "project"])}>
        {currentProject?.name}
      </p>
    </div>
  );
};

export default memo(Breadcrumb);
