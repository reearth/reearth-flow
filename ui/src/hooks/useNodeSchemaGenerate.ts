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
        description: t(
          "The custom name that is shown on the node. If not set, the official name will be used.",
        ),
        format: "text",
        default: nodeMeta.officialName,
      },
    },
  };

  const noteCustomizationSchema: RJSFSchema = {
    ...baseCustomizationSchema,
    properties: {
      ...baseCustomizationSchema.properties,
      content: {
        type: "string",
        format: "textarea",
        title: t("Content"),
        description: t("The content shown on the note node"),
      },
      backgroundColor: {
        type: "string",
        format: "color",
        default: "#212121",
        title: t("Background Color"),
        description: t("The background color shown on the note node"),
      },
      textColor: {
        type: "string",
        format: "color",
        default: "#fafafa",
        title: t("Text Color"),
        description: t("The text color shown on the note node"),
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
        description: t("The background color shown on the batch node"),
      },
      textColor: {
        type: "string",
        format: "color",
        title: t("Text Color"),
        description: t("The text color shown on the batch node"),
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

      case "subworkflow":
        resultAction = {
          ...nodeMeta,
          name: t("Subworkflow node"),
          description: t(
            "Subworkflow nodes are for creating subworkflows and grouping those workflows together.",
          ),
          type: "subworkflow",
          customizations: baseCustomizationSchema,
          inputPorts: ["input"],
          outputPorts: ["output"],
          categories: ["organization"],
          builtin: true,
        };
        break;
    }
  }

  return { action: resultAction };
};
