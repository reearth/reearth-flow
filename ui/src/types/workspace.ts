import type { Workspace as WorkspaceGraphqlType } from "@flow/lib/gql/__gen__/graphql";

import type { Member } from "./member";
import type { Project } from "./project";

export type Workspace = Pick<WorkspaceGraphqlType, "id" | "name" | "personal"> & {
  members: Member[] | undefined;
  projects: Project[] | undefined;
};
