import { memo } from "react";

import { Edge, Node } from "@flow/types";

import {
  ActionBar,
  CanvasActionBar,
  Toolbox,
  Breadcrumb,
  Infobar,
} from "./components";

type OverlayUIProps = {
  hoveredDetails: Node | Edge | undefined;
  children?: React.ReactNode;
};

const OverlayUI: React.FC<OverlayUIProps> = ({
  hoveredDetails,
  children: canvas,
}) => {
  // const { devMode } = config();
  return (
    <div className="relative flex flex-1 flex-col">
      {/* {devMode && <DevTools />} */}
      {canvas}
      <Breadcrumb />
      <Toolbox />
      <ActionBar />
      <CanvasActionBar />
      <Infobar hoveredDetails={hoveredDetails} />
    </div>
  );
};

export default memo(OverlayUI);
