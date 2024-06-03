import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { CreateProjectMutation, useGraphQLContext } from "@flow/lib/gql";

import { graphql } from "../__gen__";

graphql(`
  mutation CreateProject($input: CreateProjectInput!) {
    createProject(input: $input) {
      project {
        id
      }
    }
  }
`);

graphql(`
  query GetProjects($workspaceId: ID!, $first: Int!) {
    projects(workspaceId: $workspaceId, first: $first) {
      nodes {
        name
        id
      }
    }
  }
`);

graphql(`
  mutation UpdateProject($input: UpdateProjectInput!) {
    updateProject(input: $input) {
      project {
        id
        name
      }
    }
  }
`);

graphql(`
  mutation DeleteProject($input: DeleteProjectInput!) {
    deleteProject(input: $input) {
      projectId
    }
  }
`);

export enum ProjectQueryKeys {
  GetProjects = "getProjects",
}

type mutationInput = {
  onSuccess?: () => void;
  onError?: () => void;
};

export const useCreateProjectMutation = ({
  onSuccess,
  onError,
}: {
  onSuccess: (data: CreateProjectMutation) => void;
  onError: () => void;
}) => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: graphQLContext?.CreateProject,
    onError: onError,
    onSuccess: data => {
      queryClient.invalidateQueries({ queryKey: [ProjectQueryKeys.GetProjects] });
      onSuccess && onSuccess(data);
    },
  });
};

export const useGetProjectQuery = ({
  workspaceId,
  first,
}: {
  workspaceId: string;
  first: number;
}) => {
  const graphQLContext = useGraphQLContext();

  const input = {
    workspaceId,
    first,
  };

  const { data, ...rest } = useQuery({
    queryKey: [ProjectQueryKeys.GetProjects],
    queryFn: async () => graphQLContext?.GetProjects(input),
  });

  return { data, ...rest };
};

export const useUpdateProjectMutation = ({ onSuccess, onError }: mutationInput) => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: graphQLContext?.UpdateProject,
    onError: onError,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [ProjectQueryKeys.GetProjects] });
      onSuccess && onSuccess();
    },
  });
};

export const useDeleteProjectMutation = ({ onSuccess, onError }: mutationInput) => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: graphQLContext?.DeleteProject,
    onError: onError,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [ProjectQueryKeys.GetProjects] });
      onSuccess && onSuccess();
    },
  });
};
