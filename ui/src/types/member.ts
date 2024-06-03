import { IntegrationMember } from "./integration";
import { User } from "./user";

export type Role = "reader" | "writer" | "admin";

export type UserMember = {
  userId: string;
  role: Role;
  user: User;
};

export type Member = UserMember | IntegrationMember;
