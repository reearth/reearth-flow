import { RJSFSchema } from "@rjsf/utils";

import { useT } from "../i18n";

const useNodeSchemaGenerate = (
  nodeType: string,
  officialName: string,
): RJSFSchema => {
  const t = useT();

  const baseSchema: RJSFSchema = {
    type: "object",
    properties: {
      customName: {
        type: "string",
        title: t("Custom Name"),
        format: "text",
        default: officialName,
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

  switch (nodeType) {
    case "subworkflow":
      return {
        ...baseSchema,
        properties: {
          ...baseSchema.properties,
        },
      };
    case "batch":
      return batchNodeCustomizationSchema;
    case "note":
      return noteNodeCustomizationSchema;
    default:
      return baseSchema;
  }
};

export default useNodeSchemaGenerate;
