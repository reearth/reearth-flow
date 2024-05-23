import { User } from "@auth0/auth0-react";

export type AuthHook = {
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  getAccessToken: () => Promise<string>;
  login: () => void;
  logout: () => void;
  user: User | undefined;
};
