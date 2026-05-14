import { WorkspaceFragment } from "@flow/lib/gql/__gen__/graphql";

export const mockWorkspaces: WorkspaceFragment[] = [
  {
    id: "workspace-1",
    name: "Personal Workspace",
    personal: true,
    members: [
      {
        userId: "user-1",
        role: "owner",
        user: null,
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
        role: "owner",
        user: null,
      },
      {
        userId: "user-2",
        role: "maintainer",
        user: null,
      },
      {
        userId: "user-3",
        role: "writer",
        user: null,
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
        role: "owner",
        user: null,
      },
      {
        userId: "user-1",
        role: "maintainer",
        user: null,
      },
      {
        userId: "user-5",
        role: "reader",
        user: null,
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
        role: "owner",
        user: null,
      },
      {
        userId: "user-2",
        role: "writer",
        user: null,
      },
    ],
  },
];
