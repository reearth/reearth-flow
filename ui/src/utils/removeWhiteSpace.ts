export const removeWhiteSpace = (str: string) => {
  return str.split(/\s+/).join(""); // Don't allow white space in the name
};
