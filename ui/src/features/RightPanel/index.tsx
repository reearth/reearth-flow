import { X } from "@phosphor-icons/react";
import { MouseEvent, useCallback } from "react";

import { IconButton } from "@flow/components";
import { Node } from "@flow/types";

import { ParamEditor } from "../Editor/components";

type Props = {
  selected?: Node;
};

const RightPanel: React.FC<Props> = ({ selected }) => {
  // @KaWaite This is a little hacky, but it works. We need to dispatch a click event to the react-flow__pane
  // to unlock the node when user wants to close the right panel.
  const handleClick = useCallback((e: MouseEvent) => {
    e.stopPropagation();

    // react-flow__pane is the classname of the div inside react-flow that has the click event
    // https://github.com/xyflow/xyflow/blob/71db83761c245493d44e74311e10cc6465bf8387/packages/react/src/container/Pane/index.tsx#L249
    const paneElement = document.getElementsByClassName("react-flow__pane")[0];
    if (!paneElement) return;
    const clickEvent = new Event("click", { bubbles: true, cancelable: true });
    paneElement.dispatchEvent(clickEvent);
  }, []);

  return (
    <>
      <div
        id="right-panel-overlay"
        className="bg-black/25 border-l border-zinc-700 fixed right-0 h-full w-full"
        style={{
          transform: `translateX(${selected ? "0" : "100%"})`,
          transitionDuration: "0ms",
          transitionProperty: "transform",
        }}>
        <div className="flex justify-end fixed right-[350px] p-4 z-[1]">
          <IconButton
            className="relative before:absolute before:top-0 before:bottom-0 before:right-0 before:-z-1 before:content-[''] before:bg-green-500"
            icon={<X className="w-[30px] h-[30px]" weight="thin" />}
            onClick={handleClick}
          />
        </div>
      </div>
      <div
        id="right-panel"
        className="flex bg-zinc-800 border-l border-zinc-700 fixed right-0 h-full w-[350px] transition-all"
        style={{
          transform: `translateX(${selected ? "0" : "100%"})`,
          transitionDuration: selected ? "500ms" : "300ms",
          transitionProperty: "transform",
          transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
        }}>
        <div className="bg-zinc-900/50 w-full h-full py-4 pl-4 pr-2">
          {selected && (
            <ParamEditor nodeId={selected.id} nodeMeta={selected.data} nodeType="transformer" />
          )}
        </div>
      </div>
    </>
  );
};

export default RightPanel;
