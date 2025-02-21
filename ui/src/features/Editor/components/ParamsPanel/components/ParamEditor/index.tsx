import { RJSFSchema } from "@rjsf/utils";
import { JSONSchema7Definition } from "json-schema";
import { memo, useMemo, useState } from "react";

import { SchemaForm, Button } from "@flow/components";
import { patchAnyOfType } from "@flow/components/SchemaForm/patchSchemaTypes";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import { batchNodeAction, noteNodeAction } from "@flow/lib/reactFlow";
import type { NodeData } from "@flow/types";

type Props = {
  nodeId: string;
  nodeMeta: NodeData;
  nodeType: string;
  nodeParameters?: unknown; // TODO: define type
  onSubmit: (nodeId: string, data: any) => void;
};

// const actionButtonClasses = "border h-[25px]";

const ParamEditor: React.FC<Props> = ({
  nodeId,
  nodeMeta,
  // nodeType,
  // nodeParameters = [{ id: "param1", name: "Param 1", value: "Value 1", type: "string"}],
  onSubmit,
}) => {
  const t = useT();
  const { useGetActionById } = useAction(i18n.language);
  let { action } = useGetActionById(nodeMeta.officialName);

  if (!action) {
    switch (nodeMeta.officialName) {
      case "batch":
        action = {
          ...nodeMeta,
          ...batchNodeAction,
        };
        break;

      case "note":
        action = {
          ...nodeMeta,
          ...noteNodeAction,
        };
        break;
      default:
        action = undefined;
    }
  }
  // This is a patch for the `anyOf` type in JSON Schema.
  const patchedSchema = useMemo<RJSFSchema | undefined>(
    () =>
      action?.parameter
        ? patchAnyOfType(action.parameter as JSONSchema7Definition)
        : undefined,
    [action?.parameter],
  );

  const [updatedParams, setUpdatedParams] = useState(nodeMeta.params);

  const handleChange = (data: any) => setUpdatedParams(data);

  const handleSubmit = () => onSubmit(nodeId, updatedParams);

  return (
    <div className="flex h-full flex-col gap-4">
      {/* <div className="mb-3 flex justify-between gap-4"> */}
      {/* <div className="flex gap-2">
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
        </div> */}
      {/* </div> */}
      {/* <Tabs defaultValue="params" className="w-full"> */}
      <div className="flex justify-between gap-2">
        <p className="text-lg dark:font-thin">{t("Parameters")}</p>
        <Button onClick={handleSubmit}>{t("Submit")}</Button>
      </div>
      {/* <TabsTrigger className="flex-1" value="data">
            {t("Node data")}
          </TabsTrigger> */}

      {/* <TabsContent value="params"> */}
      <div className="min-h-0 overflow-scroll rounded border bg-card px-2">
        {!action?.parameter && <p>{t("No Parameters Available")}</p>}
        {action && (
          <SchemaForm
            schema={patchedSchema}
            defaultFormData={nodeMeta.params}
            onChange={handleChange}
            // onSubmit={handleSubmit}
          />
        )}
      </div>
      {/* </TabsContent> */}
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
      {/* </Tabs> */}
    </div>
  );
};

export default memo(ParamEditor);
