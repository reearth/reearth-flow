export const extractDescriptions = (schemaObj: any) => {
  if (!schemaObj || typeof schemaObj !== "object") return {};
  const descriptions: Record<string, unknown> = {};

  if (schemaObj.properties) {
    for (const [key, value] of Object.entries(schemaObj.properties)) {
      if (typeof value === "object" && value !== null) {
        let title = key;
        if ("title" in value && typeof value.title === "string") {
          title = value.title;
        }
        if ("description" in value) {
          descriptions[title] = value.description;
        }
      }
    }
  }

  return descriptions;
};
