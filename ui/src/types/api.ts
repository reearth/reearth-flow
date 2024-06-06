// Common parameters when an API request is made
export type ApiResponse = {
  isError: boolean;
  isSuccess: boolean;
  isPending: boolean;
  error: unknown;
};
