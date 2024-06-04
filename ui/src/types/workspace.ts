import type { Member, Project } from "@flow/types";

export type Workspace = {
  id: string;
  name: string;
  personal: boolean;
  members?: Member[];
  projects?: Project[];
};
