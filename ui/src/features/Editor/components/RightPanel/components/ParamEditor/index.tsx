import {
  ArrowLeft,
  ArrowRight,
  CornersIn,
  CornersOut,
  FrameCorners,
  MagnifyingGlassMinus,
  MagnifyingGlassPlus,
} from "@phosphor-icons/react";
import { useReactFlow } from "@xyflow/react";
import { memo } from "react";

import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  IconButton,
  Label,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  SchemaForm,
} from "@flow/components";
import { useFullscreen } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import type { NodeData } from "@flow/types";

type Props = {
  nodeId: string;
  nodeMeta: NodeData;
  nodeType: string;
  nodeParameters?: unknown; // TODO: define type
};

const actionButtonClasses = "border h-[25px]";

const ParamEditor: React.FC<Props> = ({
  nodeId,
  nodeMeta,
  nodeType,
  // nodeParameters = [{ id: "param1", name: "Param 1", value: "Value 1", type: "string"}],
}) => {
  const t = useT();
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { isFullscreen, handleFullscreenToggle } = useFullscreen();

  const { useGetActionById } = useAction();

  // For action nodes, nodeMeta.name is always defined. Only actions can open
  // the ParamsEditor, so this is for the TS error to go away
  const { action } = useGetActionById(nodeMeta.name ?? "");

  const handleFitView = () =>
    fitView({ nodes: [{ id: nodeId }], duration: 400 });

  return (
    <div>
      <div className="mb-3 flex justify-between gap-4">
        <div className="flex gap-2">
          <IconButton
            className={actionButtonClasses}
            icon={<ArrowLeft />}
            tooltipText="Previous selection"
          />
          <IconButton
            className={actionButtonClasses}
            icon={<ArrowRight />}
            tooltipText="Next selection"
          />
        </div>
        <div className="flex gap-2">
          <IconButton
            className={actionButtonClasses}
            icon={isFullscreen ? <CornersIn /> : <CornersOut />}
            tooltipText={
              isFullscreen ? t("Exit fullscreen") : t("Enter fullscreen")
            }
            onClick={handleFullscreenToggle}
          />
          <IconButton
            className={actionButtonClasses}
            icon={<FrameCorners className="w-[14px]" />}
            tooltipText="Fit view to selection"
            onClick={handleFitView}
          />
          <IconButton
            className={actionButtonClasses}
            icon={<MagnifyingGlassMinus />}
            tooltipText="Zoom out"
            onClick={() => zoomOut({ duration: 400 })}
          />
          <IconButton
            className={actionButtonClasses}
            icon={<MagnifyingGlassPlus />}
            tooltipText="Zoom in"
            onClick={() => zoomIn({ duration: 400 })}
          />
        </div>
      </div>
      <Tabs defaultValue="params" className="w-full">
        <TabsList className="flex">
          <TabsTrigger className="flex-1" value="params">
            {t("Parameters")}
          </TabsTrigger>
          <TabsTrigger className="flex-1" value="data">
            {t("Node data")}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="params">
          <div className="rounded border bg-card p-3">
            {action && <SchemaForm schema={action.parameter} />}
          </div>
        </TabsContent>
        <TabsContent value="data">
          <Card className="bg-transparent">
            <CardHeader>
              <CardTitle>Node data</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="space-y-1">
                <Label htmlFor="transformerId">Node</Label>
                <p className="ml-2">{nodeMeta.name}</p>
              </div>
              <div className="space-y-1">
                <Label htmlFor="inputs">Inputs</Label>
                <p className="ml-2">{nodeMeta.inputs?.join(", ") ?? "N/A"}</p>
              </div>
              <div className="space-y-1">
                <Label htmlFor="outputs">Outputs</Label>
                <p className="ml-2">{nodeMeta.outputs?.join(", ") ?? "N/A"}</p>
              </div>
              {nodeType === "transformer" && (
                <div className="space-y-1">
                  <Label htmlFor="transformerId">TransformerId</Label>
                  <p className="ml-2">{nodeMeta.transformerId ?? "N/A"}</p>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default memo(ParamEditor);
