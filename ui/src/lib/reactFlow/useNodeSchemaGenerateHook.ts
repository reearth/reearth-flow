import { RJSFSchema } from "@rjsf/utils";
import { useTranslation } from "react-i18next"; // Assuming you're using react-i18next

const useNodeSchemaGenerate = (
  nodeType: string,
  officialName: string,
): RJSFSchema => {
  const { t } = useTranslation();

  // General Node Schema
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
    type: "object",
    properties: {
      customName: { type: "string", title: t("Custom Name") },
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
    type: "object",
    properties: {
      customName: { type: "string", title: t("Custom Name") },
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
    case "reader":
      return {
        ...baseSchema,
        properties: {
          ...baseSchema.properties,
        },
      };
    case "writer":
      return {
        ...baseSchema,
        properties: {
          ...baseSchema.properties,
        },
      };
    case "transformer":
      return {
        ...baseSchema,
        properties: {
          ...baseSchema.properties,
        },
      };
    case "subworkflow":
      return {
        ...baseSchema,
        properties: {
          ...baseSchema.properties,
        },
      };
    case "batch":
      return {
        ...batchNodeCustomizationSchema,
        properties: {
          ...batchNodeCustomizationSchema.properties,
        },
      };
    case "note":
      return {
        ...noteNodeCustomizationSchema,
        properties: {
          ...noteNodeCustomizationSchema.properties,
        },
      };
    default:
      return baseSchema;
  }
};

export default useNodeSchemaGenerate;
