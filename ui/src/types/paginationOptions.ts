export type PaginationOptions = {
  page: number;
  pageSize?: number;
  orderBy?: string;
  orderDir: OrderDirection;
};

export enum OrderDirection {
  Asc = "ASC",
  Desc = "DESC",
}
