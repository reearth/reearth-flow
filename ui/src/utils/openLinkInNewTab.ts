export const openLinkInNewTab = (url: string) => {
  const openLink = () => window.open(url, "_blank", "noopener");
  return openLink;
};
