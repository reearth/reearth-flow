import { ArrowLeft, ArrowRight } from "@phosphor-icons/react";
import { memo } from "react";

import { IconButton, Tabs, TabsContent, SchemaForm } from "@flow/components";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import type { NodeData } from "@flow/types";

type Props = {
  nodeId: string;
  nodeMeta: NodeData;
  nodeType: string;
  nodeParameters?: unknown; // TODO: define type
  onSubmit: (nodeId: string, data: any) => void;
};

const actionButtonClasses = "border h-[25px]";

const ParamEditor: React.FC<Props> = ({
  nodeId,
  nodeMeta,
  // nodeType,
  // nodeParameters = [{ id: "param1", name: "Param 1", value: "Value 1", type: "string"}],
  onSubmit,
}) => {
  const t = useT();

  const { useGetActionById } = useAction();
  const { action } = useGetActionById(nodeMeta.officialName);

  const handleSubmit = (data: any) => onSubmit(nodeId, data);
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
        <div className="flex flex-col gap-2">
          <p className="text-lg dark:font-thin">{t("Parameters")}</p>
        </div>
        {/* <TabsTrigger className="flex-1" value="data">
            {t("Node data")}
          </TabsTrigger> */}

        <TabsContent value="params">
          <div className="rounded border bg-card p-3">
            {!action?.parameter && <p>{t("No Parameters Available")}</p>}
            {action && (
              <SchemaForm
                schema={action.parameter}
                defaultFormData={nodeMeta.params}
                onSubmit={handleSubmit}
              />
            )}
          </div>
        </TabsContent>
        {/* <TabsContent value="data">
          <Card className="bg-transparent">
            <CardHeader>
              <CardTitle>Node data</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="space-y-1">
                <Label htmlFor="transformerId">Node</Label>
                <p className="ml-2">
                  {nodeMeta.customName || nodeMeta.officialName}
                </p>
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
        </TabsContent> */}
      </Tabs>
    </div>
  );
};

export default memo(ParamEditor);
