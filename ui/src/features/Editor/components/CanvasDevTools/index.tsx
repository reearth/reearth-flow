import {
  useState,
  type Dispatch,
  type SetStateAction,
  type ReactNode,
  HTMLAttributes,
} from "react";

import { NodeInspector, ChangeLogger, ViewportLogger } from "./components";

import "./style.css";

export default function DevTools() {
  const [nodeInspectorActive, setNodeInspectorActive] = useState(true);
  const [changeLoggerActive, setChangeLoggerActive] = useState(true);
  const [viewportLoggerActive, setViewportLoggerActive] = useState(true);

  return (
    <div className="react-flow__devtools">
      <div className="absolute top-0 left-[50px]">
        <DevToolButton
          setActive={setNodeInspectorActive}
          active={nodeInspectorActive}
          title="Toggle Node Inspector">
          Node Inspector
        </DevToolButton>
        <DevToolButton
          setActive={setChangeLoggerActive}
          active={changeLoggerActive}
          title="Toggle Change Logger">
          Change Logger
        </DevToolButton>
        <DevToolButton
          setActive={setViewportLoggerActive}
          active={viewportLoggerActive}
          title="Toggle Viewport Logger">
          Viewport Logger
        </DevToolButton>
      </div>
      {changeLoggerActive && <ChangeLogger />}
      {nodeInspectorActive && <NodeInspector />}
      {viewportLoggerActive && <ViewportLogger />}
    </div>
  );
}

function DevToolButton({
  active,
  setActive,
  children,
  ...rest
}: {
  active: boolean;
  setActive: Dispatch<SetStateAction<boolean>>;
  children: ReactNode;
} & HTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      onClick={() => {
        console.log("CLICKED");
        setActive((a) => !a);
      }}
      className={active ? "active" : ""}
      {...rest}>
      {children}
    </button>
  );
}
