export type Action = {
  name: string;
  description: string;
  type: string;
  categories: string[];
};

export type Segregated = {
  [subKey: string]: Action[];
};
