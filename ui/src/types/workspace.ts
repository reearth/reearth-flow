import type { Workspace as WorkspaceGraphqlType } from "@flow/lib/gql";

import type { Member } from "./member";
import type { Project } from "./project";

export type Workspace = Pick<WorkspaceGraphqlType, "id" | "name" | "personal"> & {
  members?: Member[];
  projects?: Project[];
};
