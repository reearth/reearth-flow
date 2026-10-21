import type { WorkspaceFragment } from "@flow/lib/gql/__gen__/graphql";

export const mockWorkspaces: WorkspaceFragment[] = [
  {
    id: "workspace-1",
    name: "Personal Workspace",
    alias: "personal",
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
    alias: "development-team",
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
    alias: "analytics-project",
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
    alias: "design-studio",
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
  {
    id: "workspace-5",
    name: "Free Team",
    alias: "free-team",
    personal: false,
    members: [
      {
        userId: "user-1",
        role: "owner",
        user: null,
      },
    ],
  },
];

// PROVISIONAL (free-tier policy, not yet confirmed): team workspaces on the
// free plan, which Flow does not let users work in.
export const mockFreeTeamWorkspaceIds = new Set(["workspace-5"]);
