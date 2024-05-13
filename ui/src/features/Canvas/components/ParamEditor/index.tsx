import {
  ArrowLeftIcon,
  ArrowRightIcon,
  EnterFullScreenIcon,
  ExitFullScreenIcon,
  ZoomInIcon,
  ZoomOutIcon,
} from "@radix-ui/react-icons";
import { useReactFlow } from "reactflow";

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  CenterIcon,
  IconButton,
  Input,
  Label,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { useFullscreen } from "@flow/hooks";
import { useT } from "@flow/providers";
import type { NodeData } from "@flow/types";

type Props = {
  nodeId: string;
  nodeMeta: NodeData;
  nodeType: string;
  nodeParameters?: any; // TODO: define type
};

const actionButtonClasses = "border border-zinc-500 h-[25px]";

const ParamEditor: React.FC<Props> = ({
  nodeId,
  nodeMeta,
  nodeType,
  // nodeParameters = [{ id: "param1", name: "Param 1", value: "Value 1", type: "string"}],
}) => {
  const t = useT();
  const { zoomIn, zoomOut, fitView } = useReactFlow();
  const { isFullscreen, handleFullscreenToggle } = useFullscreen();
  console.log("reactFlowInstance", nodeId);

  const handleFitView = () => fitView({ nodes: [{ id: nodeId }], duration: 400 });
  //   reactFlowInstance.fitView({ nodes: [{ id: nodeId }] });

  return (
    <div>
      <div className="flex gap-4 mb-3 justify-between">
        <div className="flex gap-2">
          <IconButton
            className={actionButtonClasses}
            icon={<ArrowLeftIcon />}
            tooltipText="Previous selection"
          />
          <IconButton
            className={actionButtonClasses}
            icon={<ArrowRightIcon />}
            tooltipText="Next selection"
          />
        </div>
        <div className="flex gap-2">
          <IconButton
            className={actionButtonClasses}
            icon={isFullscreen ? <ExitFullScreenIcon /> : <EnterFullScreenIcon />}
            tooltipText={isFullscreen ? t("Exit fullscreen") : t("Enter fullscreen")}
            onClick={handleFullscreenToggle}
          />
          <IconButton
            className={actionButtonClasses}
            icon={<CenterIcon className="w-[14px]" />}
            tooltipText="Fit view to selection"
            onClick={handleFitView}
          />
          <IconButton
            className={actionButtonClasses}
            icon={<ZoomOutIcon />}
            tooltipText="Zoom out"
            onClick={() => zoomOut({ duration: 400 })}
          />
          <IconButton
            className={actionButtonClasses}
            icon={<ZoomInIcon />}
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
          <Card className="bg-transparent text-zinc-300 border-zinc-700">
            <CardHeader>
              <CardTitle>{t("Parameter Editor")}</CardTitle>
              <CardDescription>
                {t("Make changes to your account here. Click save when youre done.")}
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="space-y-1">
                <Label htmlFor="username">Name</Label>
                <Input id="username" placeholder="Enter city name" />
              </div>
              <div className="space-y-1">
                <Label htmlFor="username">Longitude</Label>
                <Input id="username" type="number" placeholder="Enter longitude" />
              </div>
              <div className="space-y-1">
                <Label htmlFor="username">Latitude</Label>
                <Input id="username" type="number" placeholder="Enter latitude" />
              </div>
              <div className="space-y-1">
                <Label htmlFor="name">Source</Label>
                <Select>
                  <SelectTrigger>
                    <SelectValue placeholder="Select data source" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="light">Asset1</SelectItem>
                    <SelectItem value="dark">Asset2</SelectItem>
                    <SelectItem value="system">Asset3</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
            <CardFooter>
              <Button className="bg-zinc-700">Save changes</Button>
            </CardFooter>
          </Card>
        </TabsContent>
        <TabsContent value="data">
          <Card className="bg-transparent text-zinc-300 border-zinc-700">
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

export { ParamEditor };
