import type { CmsItemJobOrigin } from "@flow/types";

export const cmsItemPath = (item: CmsItemJobOrigin): string =>
  [item.projectName, item.modelName, item.itemTitle].join(" › ");

export const cmsItemUrl = (
  cmsUrl: string | undefined,
  workspaceId: string,
  item: CmsItemJobOrigin,
): string | undefined => {
  if (!cmsUrl) return undefined;
  const segments = [
    "workspace",
    workspaceId,
    "project",
    item.projectId,
    "content",
    item.modelId,
    "details",
    item.itemId,
  ].map(encodeURIComponent);
  return `${cmsUrl.replace(/\/+$/, "")}/${segments.join("/")}`;
};
