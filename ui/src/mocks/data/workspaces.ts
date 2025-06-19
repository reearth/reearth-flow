// Mock workspaces data

export type MockWorkspace = {
  id: string;
  name: string;
  personal: boolean;
  members: MockWorkspaceMember[];
  createdAt: string;
};

export type MockWorkspaceMember = {
  userId: string;
  role: "OWNER" | "MAINTAINER" | "WRITER" | "READER";
};

export const mockWorkspaces: MockWorkspace[] = [
  {
    id: "workspace-1",
    name: "Personal Workspace",
    personal: true,
    members: [
      {
        userId: "user-1",
        role: "OWNER",
      },
    ],
    createdAt: "2024-01-01T00:00:00Z",
  },
  {
    id: "workspace-2",
    name: "Development Team",
    personal: false,
    members: [
      {
        userId: "user-1",
        role: "OWNER",
      },
      {
        userId: "user-2",
        role: "MAINTAINER",
      },
      {
        userId: "user-3",
        role: "WRITER",
      },
    ],
    createdAt: "2024-01-02T00:00:00Z",
  },
  {
    id: "workspace-3",
    name: "Analytics Project",
    personal: false,
    members: [
      {
        userId: "user-4",
        role: "OWNER",
      },
      {
        userId: "user-1",
        role: "MAINTAINER",
      },
      {
        userId: "user-5",
        role: "READER",
      },
    ],
    createdAt: "2024-01-03T00:00:00Z",
  },
  {
    id: "workspace-4",
    name: "Design Studio",
    personal: false,
    members: [
      {
        userId: "user-3",
        role: "OWNER",
      },
      {
        userId: "user-2",
        role: "WRITER",
      },
    ],
    createdAt: "2024-01-04T00:00:00Z",
  },
];
