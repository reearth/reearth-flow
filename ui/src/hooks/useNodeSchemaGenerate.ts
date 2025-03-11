import { RJSFSchema } from "@rjsf/utils";

import { useT } from "@flow/lib/i18n";
import type { Action, NodeData } from "@flow/types";

export default (
  nodeType: string,
  nodeMeta: NodeData,
  action?: Action,
): { action?: Action } => {
  const t = useT();

  const baseCustomizationSchema: RJSFSchema = {
    type: "object",
    properties: {
      customName: {
        type: "string",
        title: t("Custom Name"),
        format: "text",
        default: nodeMeta.officialName,
      },
    },
  };

  const noteCustomizationSchema: RJSFSchema = {
    ...baseCustomizationSchema,
    properties: {
      ...baseCustomizationSchema.properties,
      content: { type: "string", format: "textarea", title: t("Content") },
      backgroundColor: {
        type: "string",
        format: "color",
        default: "#212121",
        title: t("Background Color"),
      },
      textColor: {
        type: "string",
        format: "color",
        default: "#fafafa",
        title: t("Text Color"),
      },
    },
  };

  const batchCustomizationSchema: RJSFSchema = {
    ...baseCustomizationSchema,
    properties: {
      ...baseCustomizationSchema.properties,
      backgroundColor: {
        type: "string",
        format: "color",
        default: "#323236",
        title: t("Background Color"),
      },
      textColor: {
        type: "string",
        format: "color",
        title: t("Text Color"),
        default: "#fafafa",
      },
    },
  };

  let resultAction = action;

  // For Nodes that are in the actions list and have params.
  if (resultAction) {
    resultAction.customizations = baseCustomizationSchema;
  }

  // For nodes such as note and batch that are not in the actions list and therefore have no params.
  if (!resultAction) {
    switch (nodeType) {
      case "batch":
        resultAction = {
          ...nodeMeta,
          name: t("Batch Node"),
          description: t(
            "Batch nodes are for grouping multiple nodes together.",
          ),
          type: "batch",
          customizations: batchCustomizationSchema,
          inputPorts: ["input"],
          outputPorts: ["output"],
          categories: ["organization"],
          builtin: true,
        };
        break;

      case "note":
        resultAction = {
          ...nodeMeta,
          name: t("Note node"),
          description: t("Note nodes are for adding notes to the canvas."),
          type: "note",
          customizations: noteCustomizationSchema,
          inputPorts: ["input"],
          outputPorts: ["output"],
          categories: ["organization"],
          builtin: true,
        };
        break;
    }
  }

  console.log("result action", resultAction);

  return { action: resultAction };
};
