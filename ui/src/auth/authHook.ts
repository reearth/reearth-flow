import { User } from "@auth0/auth0-react";

export type AuthHook = {
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  getAccessToken: () => Promise<string>;
  login: () => void;
  logout: () => void;
  // TODO: Get the user from the API
  user: User | undefined;
};
