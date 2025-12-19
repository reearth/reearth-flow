import { IntegrationMember } from "./integration";
import { User } from "./user";

export enum Role {
  Maintainer = "maintainer",
  Owner = "owner",
  Reader = "reader",
  Writer = "writer",
}

export type UserMember = {
  userId: string;
  role: Role;
  user?: User;
};

export type Member = UserMember | IntegrationMember;
