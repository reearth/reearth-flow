import { useQuery } from "@tanstack/react-query";

import type { CmsItem, CmsModel, CmsProject } from "@flow/types/cmsIntegration";
import { isDefined } from "@flow/utils";

import type { CmsExportType } from "../__gen__/graphql";
import { toCmsAsset, toCmsItem, toCmsModel, toCmsProject } from "../convert";
import { useGraphQLContext } from "../provider";

export enum CmsQueryKeys {
  GetCmsProjects = "getCmsProjects",
  GetCmsProject = "getCmsProject",
  GetCmsModels = "getCmsModels",
  GetCmsItems = "getCmsItems",
  GetCmsAsset = "getCmsAsset",
  GetCmsModelExportUrl = "getCmsModelExportUrl",
}

export const CMS_ITEMS_FETCH_RATE = 30;
export const CMS_MODELS_FETCH_RATE = 15;
export const useQueries = () => {
  const graphQLContext = useGraphQLContext();

  const useGetCmsProjectsQuery = (
    workspaceIds: [string],
    keyword?: string,
    publicOnly?: boolean,
    page?: number,
    pageSize?: number,
  ) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsProjects, workspaceIds],
      queryFn: async () => {
        const data = await graphQLContext?.GetCmsProjects({
          workspaceIds,
          keyword,
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

  const useGetCmsProjectByIdOrAliasQuery = (
    workspaceId: string,
    projectIdOrAlias: string,
  ) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsProject, workspaceId, projectIdOrAlias],
      queryFn: () =>
        graphQLContext?.GetCmsProjectByIdOrAlias({
          workspaceId,
          projectIdOrAlias,
        }),
      enabled: !!workspaceId && !!projectIdOrAlias,
      select: (data) =>
        data?.cmsProject ? toCmsProject(data.cmsProject) : undefined,
    });

  const useGetCmsModelsQuery = (
    workspaceId: string,
    projectId: string,
    page?: number,
    pageSize?: number,
  ) =>
    useQuery({
      queryKey: [
        CmsQueryKeys.GetCmsModels,
        workspaceId,
        projectId,
        page,
        pageSize,
      ],

      queryFn: async () => {
        const data = await graphQLContext?.GetCmsModels({
          workspaceId,
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
      enabled: !!workspaceId && !!projectId,
    });

  const useGetCmsItemsQuery = (
    workspaceId: string,
    projectId: string,
    modelId: string,
    keyword?: string,
    page?: number,
    pageSize?: number,
  ) =>
    useQuery({
      queryKey: [
        CmsQueryKeys.GetCmsItems,
        workspaceId,
        projectId,
        modelId,
        page,
        pageSize,
      ],
      queryFn: async () => {
        const data = await graphQLContext?.GetCmsItems({
          workspaceId,
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
      enabled: !!workspaceId && !!projectId && !!modelId,
    });

  const useGetCmsModelExportUrlQuery = (
    workspaceId: string,
    projectId: string,
    modelId: string,
    exportType: CmsExportType,
  ) =>
    useQuery({
      queryKey: [
        CmsQueryKeys.GetCmsModelExportUrl,
        workspaceId,
        projectId,
        modelId,
        exportType,
      ],
      queryFn: async () => {
        const data = await graphQLContext?.GetCmsModelExportUrl({
          workspaceId,
          projectId,
          modelId,
          exportType,
        });
        if (!data) return;

        const cmsModelExportUrl = data.cmsModelExportUrl;

        return { cmsModelExportUrl };
      },
      enabled: !!workspaceId && !!projectId && !!modelId,
    });

  const useGetCmsAssetQuery = (workspaceId: string, assetId: string) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsAsset, workspaceId, assetId],
      queryFn: () =>
        graphQLContext?.GetCmsAsset({
          workspaceId,
          assetId,
        }),
      enabled: !!workspaceId && !!assetId,
      select: (data) => {
        if (!data?.cmsAsset) {
          return undefined;
        }

        return toCmsAsset(data.cmsAsset);
      },
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
