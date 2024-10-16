export const yamlToFormData = (yaml: string, fileName?: string) => {
  const yamlBlob = new Blob([yaml], { type: "text/yaml" });
  const formData = new FormData();
  formData.append("file", yamlBlob, fileName || "untitled.yaml");
  return formData;
};
