import { X } from "@phosphor-icons/react";
import { memo, useCallback } from "react";

import { IconButton } from "@flow/components";
import { Node } from "@flow/types";

import { ParamEditor } from "./components";

type Props = {
  selected?: Node;
  onParamsSubmit: (nodeId: string, data: any) => void;
  onClose?: () => void;
};

const ParamsPanel: React.FC<Props> = ({
  selected,
  onParamsSubmit,
  onClose,
}) => {
  const handleClose = useCallback(() => {
    if (onClose) {
      onClose();
    }
  }, [onClose]);
  const handleParamsSubmit = useCallback(
    async (nodeId: string, data: any) => {
      await Promise.resolve(onParamsSubmit(nodeId, data));
      handleClose();
    },
    [onParamsSubmit, handleClose],
  );

  return (
    <>
      <div
        id="right-panel-overlay"
        className={`fixed ${selected ? "right-[350px]" : "right-0"} z-10 size-full border-l bg-black/30`}
        style={{
          transform: `translateX(${selected ? "0" : "100%"})`,
          transitionDuration: "0ms",
          transitionProperty: "transform",
        }}>
        <div className="fixed right-0 z-[1] flex justify-end p-4">
          <IconButton
            className="relative before:absolute before:inset-y-0 before:right-0 before:z-[-1] before:bg-success before:content-['']"
            icon={<X className="size-[30px]" weight="thin" />}
            onClick={handleClose}
          />
        </div>
      </div>
      <div
        id="params-panel"
        className="fixed right-0 flex h-full w-[350px] border-l bg-background transition-all"
        style={{
          transform: `translateX(${selected ? "0" : "100%"})`,
          transitionDuration: selected ? "500ms" : "300ms",
          transitionProperty: "transform",
          transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
        }}>
        <div className="size-full px-2 py-4">
          {selected && (
            <ParamEditor
              nodeId={selected.id}
              nodeMeta={selected.data}
              nodeType={selected.type}
              onSubmit={handleParamsSubmit}
            />
          )}
        </div>
      </div>
    </>
  );
};

export default memo(ParamsPanel);
