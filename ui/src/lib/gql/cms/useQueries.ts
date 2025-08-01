import { useQuery } from "@tanstack/react-query";

import { CmsItem, CmsModel, CmsProject } from "@flow/types/cmsIntegration";
import { isDefined } from "@flow/utils";

import { toCmsItem, toCmsModel, toCmsProject } from "../convert";
import { useGraphQLContext } from "../provider";

export enum CmsQueryKeys {
  GetCmsProjects = "getCmsProjects",
  GetCmsProject = "getCmsProject",
  GetCmsModels = "getCmsModels",
  GetCmsItems = "getCmsItems",
  GetCmsModelExportUrl = "getCmsModelExportUrl",
}

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();

  const useGetCmsProjectsQuery = (workspaceId: string, publicOnly?: boolean) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsProjects, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetCmsProjects({
          workspaceId,
          publicOnly: publicOnly ?? false,
        });
        if (!data) return;

        const cmsProjects: CmsProject[] = data.cmsProjects
          .filter(isDefined)
          .map((cmsProject) => toCmsProject(cmsProject));
        return { cmsProjects };
      },
      enabled: !!workspaceId,
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

  const useGetCmsModelsQuery = (projectId: string) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsModels, projectId],
      queryFn: async () => {
        const data = await graphQLContext?.GetCmsModels({
          projectId,
        });
        if (!data) return;

        const cmsModels: CmsModel[] = data.cmsModels
          .filter(isDefined)
          .map((cmsModel) => toCmsModel(cmsModel));
        return { cmsModels };
      },
      enabled: !!projectId,
    });

  const useGetCmsItemsQuery = (
    projectId: string,
    modelId: string,
    page?: number,
    pageSize?: number,
  ) =>
    useQuery({
      queryKey: [CmsQueryKeys.GetCmsItems, projectId, modelId, page, pageSize],
      queryFn: async () => {
        const data = await graphQLContext?.GetCmsItems({
          projectId,
          modelId,
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

  return {
    useGetCmsProjectsQuery,
    useGetCmsProjectByIdOrAliasQuery,
    useGetCmsModelsQuery,
    useGetCmsItemsQuery,
    useGetCmsModelExportUrlQuery,
  };
};
