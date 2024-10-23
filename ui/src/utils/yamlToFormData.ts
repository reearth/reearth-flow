export const yamlToFormData = (yaml: string, fileName?: string) => {
  const yamlFile = new File([yaml], `${fileName || "untitled"}.yaml`, {
    type: "text/yaml",
  });
  const formData = new FormData();
  formData.append("file", yamlFile);
  return formData;
};
