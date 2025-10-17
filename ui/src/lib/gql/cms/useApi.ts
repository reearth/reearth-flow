import { CmsExportType } from "../__gen__/graphql";

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

  const useGetCmsProject = (projectIdOrAlias: string) => {
    const { data, ...rest } =
      useGetCmsProjectByIdOrAliasQuery(projectIdOrAlias);
    return {
      cmsProject: data,
      ...rest,
    };
  };

  const useGetCmsModels = (
    projectId: string,
    page?: number,
    pageSize?: number,
  ) => {
    const { data, ...rest } = useGetCmsModelsQuery(projectId, page, pageSize);
    return {
      page: data,
      ...rest,
    };
  };

  const useGetCmsItems = (
    projectId: string,
    modelId: string,
    keyword?: string,
    page?: number,
    pageSize?: number,
  ) => {
    const { data, ...rest } = useGetCmsItemsQuery(
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
    projectId: string,
    modelId: string,
    exportType: CmsExportType,
  ) => {
    const { data, ...rest } = useGetCmsModelExportUrlQuery(
      projectId,
      modelId,
      exportType,
    );
    return {
      page: data,
      ...rest,
    };
  };

  const useGetCmsAsset = (assetId: string) => {
    const { data, ...rest } = useGetCmsAssetQuery(assetId);
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
