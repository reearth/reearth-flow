import { Edge, Node } from "@flow/types";

import { ActionBar, CanvasActionBar, Toolbox, Breadcrumb, Infobar } from "./components";

type OverlayUIProps = {
  hoveredDetails: Node | Edge | undefined;
  children?: React.ReactNode;
};

const OverlayUI: React.FC<OverlayUIProps> = ({ hoveredDetails, children: canvas }) => (
  <div className="relative flex flex-col flex-1">
    {canvas}
    <Breadcrumb />
    <Toolbox />
    <ActionBar />
    <CanvasActionBar />
    <Infobar hoveredDetails={hoveredDetails} />
  </div>
);

export { OverlayUI };
