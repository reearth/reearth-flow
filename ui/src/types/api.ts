// Tanstack has many properties. Declare the ones we need to use in the code
export type ApiResponse = {
  isError: boolean;
  isSuccess: boolean;
  isPending: boolean;
  error: unknown;
};
