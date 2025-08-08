import { useQuery } from "@tanstack/react-query";

import { CmsItem, CmsModel, CmsProject } from "@flow/types/cmsIntegration";
import { isDefined } from "@flow/utils";

import { toCmsAsset, toCmsItem, toCmsModel, toCmsProject } from "../convert";
import { useGraphQLContext } from "../provider";

export enum CmsQueryKeys {
  GetCmsProjects = "getCmsProjects",
  GetCmsProject = "getCmsProject",
  GetCmsModels = "getCmsModels",
  GetCmsItems = "getCmsItems",
  GetCmsModelExportUrl = "getCmsModelExportUrl",
}

export const CMS_ITEMS_FETCH_RATE = 30;
export const CMS_MODELS_FETCH_RATE = 15;
export const useQueries = () => {
  const graphQLContext = useGraphQLContext();

  const useGetCmsProjectsQuery = (
    workspaceIds: [string],
    publicOnly?: boolean,
    page?: number,
    pageSize?: number,
  ) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsProjects, workspaceIds],
      queryFn: async () => {
        const data = await graphQLContext?.GetCmsProjects({
          workspaceIds,
          publicOnly: publicOnly ?? false,
          page,
          pageSize,
        });
        if (!data) return;

        const cmsProjects: CmsProject[] = data.cmsProjects
          .filter(isDefined)
          .map((cmsProject) => toCmsProject(cmsProject));
        return { cmsProjects };
      },
      enabled: !!workspaceIds,
    });

  const useGetCmsProjectByIdOrAliasQuery = (projectIdOrAlias: string) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsProject, projectIdOrAlias],
      queryFn: () =>
        graphQLContext?.GetCmsProjectByIdOrAlias({
          projectIdOrAlias,
        }),
      enabled: !!projectIdOrAlias,
      select: (data) =>
        data?.cmsProject?.__typename === "CMSProject"
          ? toCmsProject(data.cmsProject)
          : undefined,
    });

  const useGetCmsModelsQuery = (
    projectId: string,
    page?: number,
    pageSize?: number,
  ) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsModels, projectId, page, pageSize],

      queryFn: async () => {
        const data = await graphQLContext?.GetCmsModels({
          projectId,
          page,
          pageSize,
        });
        if (!data) return;

        const {
          cmsModels: { models, totalCount },
        } = data;
        const cmsModels: CmsModel[] = models
          .filter(isDefined)
          .map((cmsModel) => toCmsModel(cmsModel));
        return { cmsModels, totalCount };
      },
      enabled: !!projectId,
    });

  const useGetCmsItemsQuery = (
    projectId: string,
    modelId: string,
    keyword?: string,
    page?: number,
    pageSize?: number,
  ) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsItems, projectId, modelId, page, pageSize],
      queryFn: async () => {
        const data = await graphQLContext?.GetCmsItems({
          projectId,
          modelId,
          keyword,
          page: page,
          pageSize: pageSize,
        });
        if (!data) return;
        const {
          cmsItems: { items, totalCount },
        } = data;
        const cmsItems: CmsItem[] = items
          .filter(isDefined)
          .map((cmsItem: CmsItem) => toCmsItem(cmsItem));
        return { cmsItems, totalCount };
      },
      enabled: !!modelId,
    });

  const useGetCmsModelExportUrlQuery = (projectId: string, modelId: string) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsModelExportUrl, modelId],
      queryFn: async () => {
        const data = await graphQLContext?.GetCmsModelExportUrl({
          projectId,
          modelId,
        });
        if (!data) return;

        const cmsModelExportUrl = data.cmsModelExportUrl;

        return { cmsModelExportUrl };
      },
      enabled: !!modelId,
    });

  const useGetCmsAssetQuery = (assetId: string) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsProject, assetId],
      queryFn: () =>
        graphQLContext?.GetCmsAsset({
          assetId,
        }),
      enabled: !!assetId,
      select: (data) =>
        data?.cmsAsset?.__typename === "CMSAsset"
          ? toCmsAsset(data.cmsAsset)
          : undefined,
    });
  return {
    useGetCmsProjectsQuery,
    useGetCmsProjectByIdOrAliasQuery,
    useGetCmsModelsQuery,
    useGetCmsItemsQuery,
    useGetCmsAssetQuery,
    useGetCmsModelExportUrlQuery,
  };
};
