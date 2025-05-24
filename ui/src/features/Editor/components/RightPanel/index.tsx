import { X } from "@phosphor-icons/react";
import { memo } from "react";
import { Doc } from "yjs";

import { IconButton } from "@flow/components";
import type { Project } from "@flow/types";

type Props = {
  contentType?: "version-history";
  onClose: () => void;
  project?: Project;
  yDoc: Doc | null;
};

const RightPanel: React.FC<Props> = ({ contentType, onClose }) => {
  return (
    <div
      id="right-panel"
      className="fixed right-0 flex h-full w-[300px] flex-col border-l bg-background transition-all"
      style={{
        transform: `translateX(${contentType ? "0" : "100%"})`,
        transitionDuration: contentType ? "500ms" : "300ms",
        transitionProperty: "transform",
        transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
      }}>
      <div className="flex items-center border-b">
        <IconButton
          className="m-1 size-[30px] shrink-0"
          icon={<X className="size-[18px]" weight="thin" />}
          onClick={onClose}
        />
      </div>
    </div>
  );
};

export default memo(RightPanel);
