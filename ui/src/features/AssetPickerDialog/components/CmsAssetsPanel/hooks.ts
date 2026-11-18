import { useMemo, useState } from "react";

import { useCms } from "@flow/lib/gql/cms";
import { useT } from "@flow/lib/i18n";

import type { CmsAssetFormat, CmsAssetSort } from "../../utils";
import {
  CMS_ASSET_FORMATS,
  cmsAssetFormat,
  filterAndSortCmsAssets,
} from "../../utils";

export const ALL = "all";

export default ({ workspaceId }: { workspaceId: string }) => {
  const t = useT();
  const { useGetCmsProjects, useGetCmsAssetsAcrossProjects } = useCms();

  const [searchTerm, setSearchTerm] = useState("");
  const [format, setFormat] = useState<CmsAssetFormat | typeof ALL>(ALL);
  const [projectId, setProjectId] = useState<string>(ALL);
  const [sort, setSort] = useState<CmsAssetSort>("createdAtDesc");
  const [layoutView, setLayoutView] = useState<"grid" | "list">("grid");

  const projectsQuery = useGetCmsProjects([workspaceId], undefined, true);
  const projects = useMemo(
    () => projectsQuery.page?.cmsProjects ?? [],
    [projectsQuery.page],
  );

  const queriedProjects = useMemo(
    () =>
      projectId === ALL ? projects : projects.filter((p) => p.id === projectId),
    [projectId, projects],
  );

  const {
    cmsAssets,
    failedProjectCount,
    hasNextPage,
    isFetching,
    isFetchingNextPage,
    fetchNextPage,
  } = useGetCmsAssetsAcrossProjects(queriedProjects);

  const publicAssets = useMemo(
    () => cmsAssets.filter((a) => a.public),
    [cmsAssets],
  );

  const projectNames = useMemo(
    () => new Map(projects.map((p) => [p.id, p.name])),
    [projects],
  );

  const formatOptions = useMemo(() => {
    const present = new Set(
      publicAssets.map((a) => cmsAssetFormat(a.filename)),
    );
    return [
      { value: ALL, label: t("All formats") },
      ...Object.entries(CMS_ASSET_FORMATS)
        .filter(([key]) => present.has(key as CmsAssetFormat))
        .map(([key, f]) => ({ value: key, label: f.label })),
      ...(present.has("other") ? [{ value: "other", label: t("Other") }] : []),
    ];
  }, [publicAssets, t]);

  const projectOptions = useMemo(
    () => [
      { value: ALL, label: t("All CMS projects") },
      ...projects.map((p) => ({ value: p.id, label: p.name })),
    ],
    [projects, t],
  );

  const sortOptions: { value: CmsAssetSort; label: string }[] = [
    { value: "createdAtDesc", label: t("Last Uploaded") },
    { value: "createdAtAsc", label: t("First Uploaded") },
    { value: "nameAsc", label: t("A To Z") },
    { value: "nameDesc", label: t("Z To A") },
    { value: "sizeAsc", label: t("Size Small to Large") },
    { value: "sizeDesc", label: t("Size Large to Small") },
  ];

  const assets = useMemo(
    () =>
      filterAndSortCmsAssets(publicAssets, {
        keyword: searchTerm,
        format,
        sort,
      }),
    [publicAssets, searchTerm, format, sort],
  );

  const formatLabelOf = (filename: string) => {
    const f = cmsAssetFormat(filename);
    return f === "other" ? t("Other") : CMS_ASSET_FORMATS[f].label;
  };

  return {
    assets,
    projectNames,
    hasProjects: projects.length > 0,
    isLoading: projectsQuery.isFetching || (isFetching && !isFetchingNextPage),
    isFetchingNextPage,
    hasNextPage,
    failedProjectCount,
    searchTerm,
    format,
    projectId,
    sort,
    layoutView,
    formatOptions,
    projectOptions,
    sortOptions,
    formatLabelOf,
    setSearchTerm,
    setFormat,
    setProjectId,
    setSort,
    setLayoutView,
    fetchNextPage,
  };
};
