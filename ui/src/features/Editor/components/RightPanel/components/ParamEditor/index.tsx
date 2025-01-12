import { ArrowLeft, ArrowRight } from "@phosphor-icons/react";
import { RJSFSchema } from "@rjsf/utils";
import { JSONSchema7Definition } from "json-schema";
import { memo, useMemo } from "react";

import { Tabs, TabsContent, SchemaForm, IconButton } from "@flow/components";
import { patchAnyOfType } from "@flow/components/SchemaForm/patchSchemaTypes";
import { batchNodeAction } from "@flow/features/Editor/components/Canvas/components/Nodes/BatchNode";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import { Node } from "@flow/types";

type Props = {
  node: Node;
  onSubmit: (nodeId: string, data: any) => void;
};

const actionButtonClasses = "border h-[25px]";

const ParamEditor: React.FC<Props> = ({ onSubmit, node }) => {
  const t = useT();
  const { useGetActionById } = useAction(i18n.language);
  let { action } = useGetActionById(node.data.officialName);

  if (!action) {
    switch (node.data.officialName) {
      case "batch":
        action = {
          ...node,
          ...batchNodeAction,
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

  const handleSubmit = (data: any) => onSubmit(node.id, data);

  return (
    <div>
      <div className="mb-3 flex justify-between gap-4">
        <div className="flex gap-2">
          <IconButton
            className={actionButtonClasses}
            icon={<ArrowLeft />}
            tooltipText={t("Previous selection")}
          />
          <IconButton
            className={actionButtonClasses}
            icon={<ArrowRight />}
            tooltipText={t("Next selection")}
          />
        </div>
      </div>
      <Tabs defaultValue="params" className="w-full">
        <div className="flex flex-col gap-2">
          <p className="text-lg dark:font-thin">{t("Parameters")}</p>
        </div>
        <TabsContent value="params">
          <div className="rounded border bg-card p-3">
            {action && (
              <SchemaForm
                schema={patchedSchema}
                defaultFormData={node.data}
                onSubmit={handleSubmit}
              />
            )}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
};

export default memo(ParamEditor);
