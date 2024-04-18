export type Workspace = {
  id: string;
  name: string;
  members: Member[] | undefined;
  projects: Project[] | undefined;
};

export type Project = {
  id: string;
  name: string;
  workflows: Workflow[] | undefined;
};

export type Member = {
  id: string;
  name: string;
};

export type Workflow = {
  id: string;
  name: string;
  // ...nodes
};
