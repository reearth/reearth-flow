export const yamlToFormData = (yaml: string, fileName?: string) => {
  const yamlFile = new File([yaml], `${fileName || "untitled"}.yml`, {
    type: "application/x-yaml",
  });
  const formData = new FormData();
  formData.append("file", yamlFile);
  return formData;
};
