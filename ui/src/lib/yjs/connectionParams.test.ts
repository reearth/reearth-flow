import { describe, expect, it, vi } from "vitest";

import { connectionParams } from "./connectionParams";

describe("connectionParams", () => {
  it("sends the user's access token on protected pages", async () => {
    const getAccessToken = vi.fn().mockResolvedValue("jwt");
    expect(
      await connectionParams({ isProtected: true, getAccessToken }),
    ).toEqual({ token: "jwt" });
  });

  it("sends the share token on a shared page", async () => {
    const getAccessToken = vi.fn();
    expect(
      await connectionParams({ shareToken: "share-1", getAccessToken }),
    ).toEqual({ shareToken: "share-1" });
    expect(getAccessToken).not.toHaveBeenCalled();
  });

  it("prefers the access token when both are available", async () => {
    const getAccessToken = vi.fn().mockResolvedValue("jwt");
    expect(
      await connectionParams({
        isProtected: true,
        shareToken: "share-1",
        getAccessToken,
      }),
    ).toEqual({ token: "jwt" });
  });

  it("sends nothing without a credential", async () => {
    expect(await connectionParams({ getAccessToken: vi.fn() })).toEqual({});
  });
});
