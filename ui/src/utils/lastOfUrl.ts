export const lastOfUrl = (url: string) => {
  const urlArray = url.split("/");
  return urlArray[urlArray.length - 1];
};
