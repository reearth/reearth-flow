import type { Member, Project } from "@flow/types";

import type { Workspace as WorkspaceGraphqlType } from "../__gen__/graphql";

export type Workspace = Pick<WorkspaceGraphqlType, "id" | "name" | "personal"> & {
  members?: Member[];
  projects?: Project[];
};
