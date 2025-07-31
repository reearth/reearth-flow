import { User as GraphqlUser } from "@flow/lib/gql/__gen__/graphql";

// Me type according to GraphQL schema
export type MockMe = {
  id: string;
  name: string;
  email: string;
  lang: string;
  auths: string[];
  myWorkspaceId: string;
};

export const mockUsers: GraphqlUser[] = [
  {
    id: "user-1",
    name: "admin",
    email: "admin@reearth.io",
    host: "reearth.io",
  },
  {
    id: "user-2",
    name: "developer",
    email: "john@reearth.io",
    host: "reearth.io",
  },
  {
    id: "user-3",
    name: "designer",
    email: "jane@reearth.io",
    host: "reearth.io",
  },
  {
    id: "user-4",
    name: "analyst",
    email: "mike@reearth.io",
    host: "reearth.io",
  },
  {
    id: "user-5",
    name: "guest",
    email: "guest@reearth.io",
  },
];

export const getCurrentUser = (): GraphqlUser => mockUsers[0];

export const getCurrentMe = (): MockMe => ({
  id: "user-1",
  name: "admin",
  email: "admin@reearth.io",
  lang: "en",
  auths: ["auth0"],
  myWorkspaceId: "workspace-1",
});
