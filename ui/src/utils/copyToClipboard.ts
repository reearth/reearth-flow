export const copyToClipboard = (value: any) => {
  navigator.clipboard
    .writeText(value)
    .then(() => {
      console.log("Text copied to clipboard! ", value);
    })
    .catch((err) => {
      console.error("Failed to copy text: ", err);
    });
};
