export default (url: string) => {
  const openLink = () => window.open(url, "_blank", "noopener");
  return openLink;
};
