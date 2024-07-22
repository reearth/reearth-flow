import { X } from "@phosphor-icons/react";
import { MouseEvent, useCallback } from "react";

import { IconButton } from "@flow/components";
import { Node } from "@flow/types";

import { ParamEditor } from "./components";

type Props = {
  selected?: Node;
};

const RightPanel: React.FC<Props> = ({ selected }) => {
  // This is a little hacky, but it works. We need to dispatch a click event to the react-flow__pane
  // to unlock the node when user wants to close the right panel. - @KaWaite
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
        className="fixed right-0 size-full border-l border-zinc-700 bg-black/25"
        style={{
          transform: `translateX(${selected ? "0" : "100%"})`,
          transitionDuration: "0ms",
          transitionProperty: "transform",
        }}>
        <div className="fixed right-[350px] z-[1] flex justify-end p-4">
          <IconButton
            className="relative before:absolute before:inset-y-0 before:right-0 before:-z-10 before:bg-green-500 before:content-['']"
            icon={<X className="size-[30px]" weight="thin" />}
            onClick={handleClick}
          />
        </div>
      </div>
      <div
        id="right-panel"
        className="fixed right-0 flex h-full w-[350px] border-l border-zinc-700 bg-background-800 transition-all"
        style={{
          transform: `translateX(${selected ? "0" : "100%"})`,
          transitionDuration: selected ? "500ms" : "300ms",
          transitionProperty: "transform",
          transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
        }}>
        <div className="size-full bg-zinc-900/50 py-4 pl-4 pr-2">
          {selected && (
            <ParamEditor nodeId={selected.id} nodeMeta={selected.data} nodeType="transformer" />
          )}
        </div>
      </div>
    </>
  );
};

export { RightPanel };
