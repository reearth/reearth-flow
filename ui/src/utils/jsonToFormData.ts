export const jsonToFormData = (json: object, fileName?: string) => {
  const jsonString = JSON.stringify(json);
  const jsonFile = new File([jsonString], `${fileName || "untitled"}.json`, {
    type: "application/json",
  });
  const formData = new FormData();
  formData.append("file", jsonFile);
  return formData;
};
