import { RJSFSchema } from "@rjsf/utils";

import type { Action, NodeData } from "@flow/types";

import { useT } from "../i18n";

const useNodeSchemaGenerate = (
  nodeType: string,
  nodeMeta: NodeData,
  action?: Action,
): { schema: RJSFSchema; action?: Action } => {
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

  const noteNodeCustomizationSchema: RJSFSchema = {
    ...baseSchema,
    properties: {
      ...baseSchema.properties,
      content: { type: "string", format: "textarea", title: t("Content") },
      textColor: {
        type: "string",
        format: "color",
        default: "#fafafa",
        title: t("Text Color"),
      },
      backgroundColor: {
        type: "string",
        format: "color",
        default: "#212121",
        title: t("Background Color"),
      },
    },
  };

  const batchNodeCustomizationSchema: RJSFSchema = {
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
    case "reader":
    case "writer":
    case "transformer":
    case "subworkflow":
      schema = {
        ...baseSchema,
        properties: {
          ...baseSchema.properties,
        },
      };
      break;
    case "batch":
      schema = batchNodeCustomizationSchema;
      break;
    case "note":
      schema = noteNodeCustomizationSchema;
      break;
    default:
      schema = baseSchema;
  }

  let resultAction = action;
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
          customization: schema,
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
          customization: schema,
        };
        break;
    }
  }

  return { schema, action: resultAction };
};

export default useNodeSchemaGenerate;
