import { User } from "./user";

export type Role = "reader" | "writer" | "admin";

export type Member = {
  userId: string;
  role: Role;
  user: User;
};

// TODO: make integration member https://github.com/reearth/reearthx/blob/5cbc45bf18eb36f78cf4d96b683d7c6bbac2161c/account/workspace.graphql
