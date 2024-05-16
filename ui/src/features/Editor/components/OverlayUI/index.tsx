import { customTransformers } from "@flow/mock_data/customTransformer";
import { Edge, Node } from "@flow/types";

import { ActionBar, CanvasActionBar, Toolbox, Breadcrumb, CanvasTabs, Infobar } from "./components";

type OverlayUIProps = {
  hoveredDetails: Node | Edge | undefined;
  children?: React.ReactNode;
};

const OverlayUI: React.FC<OverlayUIProps> = ({ hoveredDetails, children: canvas }) => (
  <div className="relative flex flex-col flex-1">
    {canvas}
    <Breadcrumb />
    <Toolbox />
    <CanvasTabs editingCustomTransformers={customTransformers} />
    <ActionBar />
    <CanvasActionBar />
    <Infobar hoveredDetails={hoveredDetails} />
  </div>
);

export { OverlayUI };
