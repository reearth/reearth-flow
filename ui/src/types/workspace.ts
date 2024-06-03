import type { Member } from "./member";
import type { Project } from "./project";

export type Workspace = {
  id: string;
  name: string;
  description?: string;
  members: Member[] | undefined;
  projects: Project[] | undefined;
  personal?: boolean;
};
