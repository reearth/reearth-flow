import { useQuery } from "@tanstack/react-query";

import {
  CreateProjectInput,
  DeleteProjectInput,
  UpdateProjectInput,
  useGraphQLContext,
} from "@flow/lib/gql";

export const useCreateProjectQuery = (workspaceId: string, name: string, description: string) => {
  const graphQLContext = useGraphQLContext();
  const input: CreateProjectInput = {
    workspaceId,
    name,
    description,
  };
  const { data, ...rest } = useQuery({
    queryKey: ["createWorkspace"],
    queryFn: async () => graphQLContext?.CreateProject({ input }),
  });

  return { data, ...rest };
};

export const useGetProjectQuery = (workspaceId: string, first: number) => {
  const input = {
    workspaceId,
    first,
  };
  const graphQLContext = useGraphQLContext();
  const { data, ...rest } = useQuery({
    queryKey: ["getWorkspace"],
    queryFn: async () => graphQLContext?.GetProjects(input),
  });

  return { data, ...rest };
};

export const useUpdateProjectQuery = (projectId: string, name: string, description: string) => {
  const graphQLContext = useGraphQLContext();
  const input: UpdateProjectInput = {
    projectId,
    name,
    description,
  };
  const { data, ...rest } = useQuery({
    queryKey: ["updateWorkspace"],
    queryFn: async () => graphQLContext?.UpdateProject({ input }),
  });

  return { data, ...rest };
};

export const useDeleteProjectQuery = (projectId: string) => {
  const graphQLContext = useGraphQLContext();
  const input: DeleteProjectInput = {
    projectId,
  };
  const { data, ...rest } = useQuery({
    queryKey: ["deleteWorkspace"],
    queryFn: async () => graphQLContext?.DeleteProject({ input }),
  });

  return { data, ...rest };
};
