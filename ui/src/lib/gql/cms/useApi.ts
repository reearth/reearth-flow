import type { CmsExportType } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export const useCms = () => {
  const {
    useGetCmsProjectsQuery,
    useGetCmsProjectByIdOrAliasQuery,
    useGetCmsModelsQuery,
    useGetCmsItemsQuery,
    useGetCmsAssetQuery,
    useGetCmsModelExportUrlQuery,
  } = useQueries();

  const useGetCmsProjects = (
    workspaceId: [string],
    keyword?: string,
    publicOnly?: boolean,
    page?: number,
    pageSize?: number,
  ) => {
    const { data, ...rest } = useGetCmsProjectsQuery(
      workspaceId,
      keyword,
      publicOnly,
      page,
      pageSize,
    );
    return {
      page: data,
      ...rest,
    };
  };

  const useGetCmsProject = (workspaceId: string, projectIdOrAlias: string) => {
    const { data, ...rest } = useGetCmsProjectByIdOrAliasQuery(
      workspaceId,
      projectIdOrAlias,
    );
    return {
      cmsProject: data,
      ...rest,
    };
  };

  const useGetCmsModels = (
    workspaceId: string,
    projectId: string,
    page?: number,
    pageSize?: number,
  ) => {
    const { data, ...rest } = useGetCmsModelsQuery(
      workspaceId,
      projectId,
      page,
      pageSize,
    );
    return {
      page: data,
      ...rest,
    };
  };

  const useGetCmsItems = (
    workspaceId: string,
    projectId: string,
    modelId: string,
    keyword?: string,
    page?: number,
    pageSize?: number,
  ) => {
    const { data, ...rest } = useGetCmsItemsQuery(
      workspaceId,
      projectId,
      modelId,
      keyword,
      page,
      pageSize,
    );
    return {
      page: data,
      ...rest,
    };
  };

  const useGetCmsModelExportUrl = (
    workspaceId: string,
    projectId: string,
    modelId: string,
    exportType: CmsExportType,
  ) => {
    const { data, ...rest } = useGetCmsModelExportUrlQuery(
      workspaceId,
      projectId,
      modelId,
      exportType,
    );
    return {
      page: data,
      ...rest,
    };
  };

  const useGetCmsAsset = (workspaceId: string, assetId: string) => {
    const { data, ...rest } = useGetCmsAssetQuery(workspaceId, assetId);
    return {
      cmsAsset: data,
      ...rest,
    };
  };

  return {
    useGetCmsProjects,
    useGetCmsProject,
    useGetCmsModels,
    useGetCmsItems,
    useGetCmsAsset,
    useGetCmsModelExportUrl,
  };
};
