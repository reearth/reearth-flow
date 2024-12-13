import { ArrowLeft, ArrowRight } from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils/lib/types";
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
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import { Node } from "@flow/types";

type Props = {
  node: Node;
  onSubmit: (nodeId: string, data: any) => void;
};

const actionButtonClasses = "border h-[25px]";

const ParamEditor: React.FC<Props> = ({ onSubmit, node }) => {
  const t = useT();

  const { id: nodeId, data: nodeMeta, type: nodeType } = node;

  const { useGetActionById } = useAction();
  const { action: schema } = useGetActionById(nodeMeta.name);

  const handleSubmit = (data: any) => onSubmit(nodeId, data);

  // TODO: Till the backend for the batch node is ready, let's use this type
  // TODO: Implemented with just en lang
  const batchNodeSchema: RJSFSchema = {
    type: "object",
    properties: {
      type: {
        type: "string",
        const: "batch",
        title: "Type",
        readOnly: true,
      },
      name: { type: "string", title: "Name", $id: "name" },
      color: {
        type: "object",
        title: "Color (RGBA hex)",
        properties: {
          titleColor: {
            type: "string",
            title: "Title Color",
          },
          transparency: {
            type: "number",
            title: "Transparency",
          },
          backgroundColor: {
            type: "string",
            title: "Background Color",
          },
          borderColor: {
            type: "string",
            title: "Border Color",
          },
        },
      },
      size: {
        type: "object",
        title: "Size",
        properties: {
          height: { type: "number", title: "Height" },
          width: { type: "number", title: "Width" },
        },
      },
    },
    required: ["name"],
  };

  const batchNodeParams = {
    type: "batch",
    name: nodeMeta.name,
    color: {
      // titleColor: "#000000",
      // transparency: 1,
      // backgroundColor: "#ffffff",
      // borderColor: "#000000",
    },
    size: {
      height: node.height,
      width: node.width,
    },
  };

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
            {nodeType == "batch" && (
              <SchemaForm
                schema={batchNodeSchema}
                defaultFormData={{ ...nodeMeta.params, ...batchNodeParams }}
                onSubmit={handleSubmit}
              />
            )}
            {schema && (
              <SchemaForm
                schema={schema.parameter}
                defaultFormData={nodeMeta.params}
                onSubmit={handleSubmit}
              />
            )}
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
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default memo(ParamEditor);
