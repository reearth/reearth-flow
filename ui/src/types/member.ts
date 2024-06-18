import { IntegrationMember } from "./integration";
import { User } from "./user";

export type Role = "MAINTAINER" | "OWNER" | "READER" | "WRITER";

export type UserMember = {
  userId: string;
  role: Role;
  user?: User;
};

export type Member = UserMember | IntegrationMember;
