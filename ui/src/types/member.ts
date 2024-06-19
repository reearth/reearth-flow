import { IntegrationMember } from "./integration";
import { User } from "./user";

export enum Role {
  Maintainer = "MAINTAINER",
  Owner = "OWNER",
  Reader = "READER",
  Writer = "WRITER",
}

export type UserMember = {
  userId: string;
  role: Role;
  user?: User;
};

export type Member = UserMember | IntegrationMember;
