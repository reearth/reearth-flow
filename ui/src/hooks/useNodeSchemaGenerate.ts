import { RJSFSchema } from "@rjsf/utils";

import { useT } from "@flow/lib/i18n";
import type { Action, NodeData } from "@flow/types";

export default (
  nodeType: string,
  nodeMeta: NodeData,
  action?: Action,
): { action?: Action } => {
  const t = useT();

  const baseSchema: RJSFSchema = {
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

  const noteNodeSchema: RJSFSchema = {
    ...baseSchema,
    properties: {
      ...baseSchema.properties,
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

  const batchNodeSchema: RJSFSchema = {
    ...baseSchema,
    properties: {
      ...baseSchema.properties,
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

  let schema: RJSFSchema;
  switch (nodeType) {
    case "batch":
      schema = batchNodeSchema;
      break;
    case "note":
      schema = noteNodeSchema;
      break;
    default:
      schema = baseSchema;
  }

  let resultAction = action;

  // For Nodes that are in the actions list and have params.
  if (resultAction) {
    resultAction = {
      ...resultAction,
      customizations: schema,
    };
  }

  // For nodes such as note and batch that are not in the actions list and therefore have no params.
  if (!resultAction) {
    switch (nodeMeta.officialName) {
      case "batch":
        resultAction = {
          ...nodeMeta,
          name: "batch",
          description: "Batch node",
          type: "batch",
          categories: ["batch"],
          inputPorts: ["input"],
          outputPorts: ["output"],
          builtin: true,
          customizations: schema,
        };
        break;

      case "note":
        resultAction = {
          ...nodeMeta,
          name: "note",
          description: "Note node",
          type: "note",
          categories: ["note"],
          inputPorts: ["input"],
          outputPorts: ["output"],
          builtin: true,
          customizations: schema,
        };
        break;
    }
  }

  return { action: resultAction };
};
