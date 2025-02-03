export type PaginationOptions = {
  pageSize?: number;
  page?: number;
  orderBy?: string;
  orderDir?: OrderDir;
};

export enum OrderDir {
  ASC = "ASC",
  DESC = "DESC",
}
