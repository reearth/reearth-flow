import { Workflow } from "./workflow";

export type Project = {
  id: string;
  name: string;
  description?: string;
  workflows: Workflow[] | undefined;
};
