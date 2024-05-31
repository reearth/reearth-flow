import { Workflow } from "./workflow";

export type Project = {
  id: string;
  name: string;
  description?: string;
  workflow: Workflow | undefined;
  createdAt: string;
  updatedAt?: string;
};
