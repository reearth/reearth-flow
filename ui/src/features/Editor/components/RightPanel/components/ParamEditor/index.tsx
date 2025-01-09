import { RJSFSchema } from "@rjsf/utils";
import { JSONSchema7Definition } from "json-schema";
import { memo, useMemo } from "react";

import { Tabs, TabsContent, SchemaForm } from "@flow/components";
import { patchAnyOfType } from "@flow/components/SchemaForm/patchSchemaTypes";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import { Node } from "@flow/types";

type Props = {
  node: Node;
  onSubmit: (nodeId: string, data: any) => void;
};

// const actionButtonClasses = "border h-[25px]";

const ParamEditor: React.FC<Props> = ({ onSubmit, node }) => {
  const t = useT();
  const { useGetActionById } = useAction(i18n.language);
  const { action } = useGetActionById(node.data.officialName);

  // This is a patch for the `anyOf` type in JSON Schema.
  const patchedSchema = useMemo<RJSFSchema | undefined>(
    () =>
      action?.parameter
        ? patchAnyOfType(action.parameter as JSONSchema7Definition)
        : undefined,
    [action?.parameter],
  );

  const handleSubmit = (data: any) => onSubmit(node.id, data);

  return (
    <div>
      <div className="mb-3 flex justify-between gap-4">
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
                schema={patchedSchema}
                defaultFormData={node.data}
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
