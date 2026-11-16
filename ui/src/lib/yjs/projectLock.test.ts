import * as Y from "yjs";

import type { YDocMetadataMap, YDocMetadataValue } from "@flow/types";

import {
  claimsLock,
  PROJECT_LOCK_KEY,
  reconnectOnProjectLockChange,
} from "./projectLock";

const at = Date.parse("2026-11-16T00:00:00.000Z");

describe("claimsLock", () => {
  test("reads a lock claim from the entry", () => {
    expect(claimsLock({ locked: true, at, sig: "s" })).toBe(true);
    expect(claimsLock({ locked: false, at, sig: "s" })).toBe(false);
  });

  test("reads anything else as no claim", () => {
    expect(claimsLock(undefined)).toBe(false);
    expect(claimsLock(true)).toBe(false);
    expect(claimsLock({ locked: "yes" })).toBe(false);
  });
});

describe("reconnectOnProjectLockChange", () => {
  const setup = (initial?: { locked: boolean; at: number; sig: string }) => {
    const doc = new Y.Doc();
    const metadata: YDocMetadataMap = doc.getMap<YDocMetadataValue>("metadata");
    if (initial) metadata.set(PROJECT_LOCK_KEY, initial);
    const provider = { synced: true, connect: vi.fn(), disconnect: vi.fn() };
    const stop = reconnectOnProjectLockChange(metadata, provider);
    const signal = (locked: boolean, when: number) =>
      metadata.set(PROJECT_LOCK_KEY, { locked, at: when, sig: "s" });
    return { metadata, provider, stop, signal };
  };

  test("reconnects when the lock changes while connected", () => {
    const { provider, signal } = setup({ locked: false, at, sig: "s" });

    signal(true, at + 1);
    expect(provider.disconnect).toHaveBeenCalledTimes(1);
    expect(provider.connect).toHaveBeenCalledTimes(1);

    signal(false, at + 2);
    expect(provider.connect).toHaveBeenCalledTimes(2);
  });

  test("does not reconnect when the server writes the same lock again", () => {
    const { provider, signal } = setup({ locked: true, at, sig: "s" });

    signal(true, at);
    expect(provider.connect).not.toHaveBeenCalled();
  });

  test("takes the entry received while syncing as the connection's state", () => {
    const { provider, signal } = setup();

    provider.synced = false;
    signal(true, at);
    expect(provider.connect).not.toHaveBeenCalled();

    provider.synced = true;
    signal(true, at);
    expect(provider.connect).not.toHaveBeenCalled();
    signal(false, at + 1);
    expect(provider.connect).toHaveBeenCalledTimes(1);
  });

  test("ignores other metadata", () => {
    const { metadata, provider } = setup({ locked: false, at, sig: "s" });

    metadata.set("layoutDirection", "Horizontal");
    expect(provider.connect).not.toHaveBeenCalled();
  });

  test("stops when cleaned up", () => {
    const { provider, stop, signal } = setup({ locked: false, at, sig: "s" });

    stop();
    signal(true, at + 1);
    expect(provider.connect).not.toHaveBeenCalled();
  });
});
