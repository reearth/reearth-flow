type User = {
  email: string;
  name: string;
  nickname: string;
  picture: string;
};

export type AuthHook = {
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  getAccessToken: () => Promise<string>;
  login: () => void;
  logout: () => void;
  user: User | undefined;
};
