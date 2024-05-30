import { useQuery } from "@tanstack/react-query";

import {
  CreateWorkspaceInput,
  DeleteWorkspaceInput,
  UpdateWorkspaceInput,
  useGraphQLContext,
} from "@flow/lib/gql";

// TODO: const graphQLContext = useGraphQLContext(); is repeated everywhere
// graphQLContext?.[ACTION]({ input }) is also repeated everywhere

export const useCreateWorkspaceQuery = (name: string) => {
  const graphQLContext = useGraphQLContext();
  const input: CreateWorkspaceInput = {
    name,
  };
  const { data, ...rest } = useQuery({
    queryKey: ["createWorkspace"],
    queryFn: async () => graphQLContext?.CreateWorkspace({ input }),
  });

  return { data, ...rest };
};

export const useGetWorkspaceQuery = () => {
  const graphQLContext = useGraphQLContext();
  const { data, ...rest } = useQuery({
    queryKey: ["getWorkspace"],
    queryFn: async () => graphQLContext?.GetWorkspaces(),
  });

  return { data, ...rest };
};

export const useUpdateWorkspaceQuery = (workspaceId: string, name: string) => {
  const graphQLContext = useGraphQLContext();
  const input: UpdateWorkspaceInput = {
    workspaceId,
    name,
  };
  const { data, ...rest } = useQuery({
    queryKey: ["updateWorkspace"],
    queryFn: async () => graphQLContext?.UpdateWorkspace({ input }),
  });

  return { data, ...rest };
};

export const useDeleteWorkspaceQuery = (workspaceId: string) => {
  const graphQLContext = useGraphQLContext();
  const input: DeleteWorkspaceInput = {
    workspaceId,
  };
  const { data, ...rest } = useQuery({
    queryKey: ["deleteWorkspace"],
    queryFn: async () => graphQLContext?.DeleteWorkspace({ input }),
  });

  return { data, ...rest };
};
