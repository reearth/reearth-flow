import type { Member } from "./member";
import type { Project } from "./project";

// TODO: Shouldn't this come from graphql?
export type Workspace = {
  id: string;
  name: string;
  members: Member[] | undefined;
  projects: Project[] | undefined;
};
