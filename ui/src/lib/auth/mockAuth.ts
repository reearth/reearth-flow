import type { AuthHook } from "./types";

// Dev-only stand-in for useAuth0Auth so mock mode can render the app without an
// identity provider.
//
// Without it, AuthProvider returns null whenever auth0Domain/auth0ClientId are
// absent, so the entire tree renders nothing and — because returning null is legal
// React — there is no error to diagnose. That silent blank page is what mock mode
// produced for anyone who followed the README.
//
// Lives here rather than under src/mocks/ on purpose: vite.config.ts declares
// ./src/mocks/** external for the production build, so app code must not import
// from it. The only call site is guarded by import.meta.env.DEV, which Vite
// statically replaces with false when building, so this is eliminated from the
// bundle rather than merely being unreachable at runtime.
export const useMockAuth = (): AuthHook => ({
  isAuthenticated: true,
  isLoading: false,
  error: null,
  // Never sent anywhere real: the mocked GraphQL layer ignores it, and the
  // websocket only forwards a token in protected mode.
  getAccessToken: async () => "mock-access-token",
  login: () => {},
  logout: () => {},
  user: {
    sub: "mock|user-1",
    name: "admin",
    email: "admin@reearth.io",
  },
});
