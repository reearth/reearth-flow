export const Theme = {
  Default: "DEFAULT",
  Dark: "DARK",
  Light: "LIGHT",
} as const;
export type Theme = (typeof Theme)[keyof typeof Theme];

export type MockUser = {
  id: string;
  name: string;
  email: string;
  host?: string;
  metadata?: {
    description?: string | null;
    website?: string | null;
    photoURL?: string | null;
    theme?: Theme;
    lang?: string;
  };
};

// Me type according to GraphQL schema
export type MockMe = {
  id: string;
  name: string;
  email: string;
  lang: string;
  auths: string[];
  myWorkspaceId: string;
};

export const mockUsers: MockUser[] = [
  {
    id: "user-1",
    name: "admin",
    email: "admin@reearth.io",
    host: "reearth.io",
    metadata: {
      description: "admin description",
      website: "https://example.com/admin",
      photoURL: "https://example.com/avatars/admin.png",
      theme: Theme.Default,
      lang: "ja",
    },
  },
  {
    id: "user-2",
    name: "developer",
    email: "john@reearth.io",
    host: "reearth.io",
    metadata: {
      description: "developer description",
      website: "https://example.com/developer",
      photoURL: "https://example.com/avatars/developer.png",
      theme: Theme.Dark,
      lang: "en",
    },
  },
  {
    id: "user-3",
    name: "designer",
    email: "jane@reearth.io",
    host: "reearth.io",
    metadata: {
      description: "designer description",
      website: "https://example.com/designer",
      photoURL: "https://example.com/avatars/designer.png",
      theme: Theme.Light,
      lang: "ja",
    },
  },
  {
    id: "user-4",
    name: "analyst",
    email: "mike@reearth.io",
    host: "reearth.io",
    metadata: {
      description: "analyst description",
      website: "https://example.com/analyst",
      photoURL: "https://example.com/avatars/analyst.png",
      theme: Theme.Light,
      lang: "en",
    },
  },
  {
    id: "user-5",
    name: "guest",
    email: "guest@reearth.io",
    metadata: {
      description: "guest description",
      website: null,
      photoURL: null,
      theme: Theme.Default,
      lang: "en",
    },
  },
];

export const getCurrentUser = (): MockUser => mockUsers[0];

export const getCurrentMe = (): MockMe => ({
  id: "user-1",
  name: "admin",
  email: "admin@reearth.io",
  lang: "en",
  auths: ["auth0"],
  myWorkspaceId: "workspace-1",
});
