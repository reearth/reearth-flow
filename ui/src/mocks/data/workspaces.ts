import {
  WorkspaceFragment,
  Role as GraphqlRole,
} from "@flow/lib/gql/__gen__/graphql";

export const mockWorkspaces: WorkspaceFragment[] = [
  {
    id: "workspace-1",
    name: "Personal Workspace",
    personal: true,
    members: [
      {
        userId: "user-1",
        role: GraphqlRole.Owner,
      },
    ],
  },
  {
    id: "workspace-2",
    name: "Development Team",
    personal: false,
    members: [
      {
        userId: "user-1",
        role: GraphqlRole.Owner,
      },
      {
        userId: "user-2",
        role: GraphqlRole.Maintainer,
      },
      {
        userId: "user-3",
        role: GraphqlRole.Writer,
      },
    ],
  },
  {
    id: "workspace-3",
    name: "Analytics Project",
    personal: false,
    members: [
      {
        userId: "user-4",
        role: GraphqlRole.Owner,
      },
      {
        userId: "user-1",
        role: GraphqlRole.Maintainer,
      },
      {
        userId: "user-5",
        role: GraphqlRole.Reader,
      },
    ],
  },
  {
    id: "workspace-4",
    name: "Design Studio",
    personal: false,
    members: [
      {
        userId: "user-3",
        role: GraphqlRole.Owner,
      },
      {
        userId: "user-2",
        role: GraphqlRole.Writer,
      },
    ],
  },
];
