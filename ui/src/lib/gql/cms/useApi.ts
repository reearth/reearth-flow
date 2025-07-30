import { useQueries } from "./useQueries";

export const useCms = () => {
  const {
    useGetCmsProjectsQuery,
    useGetCmsProjectByIdOrAliasQuery,
    useGetCmsModelsQuery,
    useGetCmsItemsQuery,
    useGetCmsModelExportUrlQuery,
  } = useQueries();

  const useGetCmsProjects = (workspaceId: string, publicOnly?: boolean) => {
    const { data, ...rest } = useGetCmsProjectsQuery(workspaceId, publicOnly);
    return {
      page: data,
      ...rest,
    };
  };

  const useGetCmsProject = (projectIdOrAlias?: string) => {
    const { data, ...rest } =
      useGetCmsProjectByIdOrAliasQuery(projectIdOrAlias);
    return {
      cmsProject: data,
      ...rest,
    };
  };

  const useGetCmsModels = (projectId: string) => {
    const { data, ...rest } = useGetCmsModelsQuery(projectId);
    return {
      page: data,
      ...rest,
    };
  };

  const useGetCmsItems = (
    projectId: string,
    modelId: string,
    page?: number,
    pageSize?: number,
  ) => {
    const { data, ...rest } = useGetCmsItemsQuery(
      projectId,
      modelId,
      page,
      pageSize,
    );
    return {
      page: data,
      ...rest,
    };
  };

  const useGetCmsModelExportUrl = (projectId: string, modelId: string) => {
    const { data, ...rest } = useGetCmsModelExportUrlQuery(projectId, modelId);
    return {
      page: data,
      ...rest,
    };
  };

  return {
    useGetCmsProjects,
    useGetCmsProject,
    useGetCmsModels,
    useGetCmsItems,
    useGetCmsModelExportUrl,
  };
};
