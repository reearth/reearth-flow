export const copyToClipboard = (value: any) => {
  navigator.clipboard
    .writeText(value)
    .then(() => {
      // The value is not logged: it can be a secret, such as an auth token.
      console.log("Text copied to clipboard!");
    })
    .catch((err) => {
      console.error("Failed to copy text: ", err);
    });
};
