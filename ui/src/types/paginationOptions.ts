export type PaginationOptions = {
  pageSize?: number;
  page?: number;
  orderBy?: string;
  orderDir?: OrderDirection;
};

export enum OrderDirection {
  Asc = "ASC",
  Desc = "DESC",
}
