import * as Y from "yjs";

import type { YDocMetadataMap, YDocMetadataValue } from "@flow/types";

import {
  PROJECT_LOCK_KEY,
  readProjectLock,
  reconnectOnProjectLockChange,
  resolveProjectLock,
} from "./projectLock";

const at = Date.parse("2026-11-16T00:00:00.000Z");
const project = (
  isLocked: boolean,
  updatedAt = new Date(at).toISOString(),
) => ({
  isLocked,
  updatedAt,
});

describe("readProjectLock", () => {
  test("reads the entry the server writes", () => {
    expect(readProjectLock({ locked: true, at })).toEqual({ locked: true, at });
  });

  test("ignores anything else", () => {
    expect(readProjectLock(undefined)).toBeUndefined();
    expect(readProjectLock(true)).toBeUndefined();
    expect(readProjectLock({ locked: "yes", at })).toBeUndefined();
    expect(readProjectLock({ locked: true })).toBeUndefined();
  });
});

describe("resolveProjectLock", () => {
  test("follows the project without a signal", () => {
    expect(resolveProjectLock(project(true), undefined)).toBe(true);
    expect(resolveProjectLock(project(false), undefined)).toBe(false);
    expect(resolveProjectLock(undefined, undefined)).toBe(false);
  });

  test("follows a signal newer than the fetched project", () => {
    expect(
      resolveProjectLock(project(false), { locked: true, at: at + 1 }),
    ).toBe(true);
    expect(
      resolveProjectLock(project(true), { locked: false, at: at + 1 }),
    ).toBe(false);
  });

  test("ignores a signal the project already has or has moved past", () => {
    expect(resolveProjectLock(project(true), { locked: true, at })).toBe(true);
    expect(
      resolveProjectLock(project(false), { locked: true, at: at - 1 }),
    ).toBe(false);
  });
});

describe("reconnectOnProjectLockChange", () => {
  const setup = (initial?: { locked: boolean; at: number }) => {
    const doc = new Y.Doc();
    const metadata: YDocMetadataMap = doc.getMap<YDocMetadataValue>("metadata");
    if (initial) metadata.set(PROJECT_LOCK_KEY, initial);
    const provider = { synced: true, connect: vi.fn(), disconnect: vi.fn() };
    const stop = reconnectOnProjectLockChange(metadata, provider);
    const signal = (locked: boolean, when: number) =>
      metadata.set(PROJECT_LOCK_KEY, { locked, at: when });
    return { metadata, provider, stop, signal };
  };

  test("reconnects when the lock changes while connected", () => {
    const { provider, signal } = setup({ locked: false, at });

    signal(true, at + 1);
    expect(provider.disconnect).toHaveBeenCalledTimes(1);
    expect(provider.connect).toHaveBeenCalledTimes(1);

    signal(false, at + 2);
    expect(provider.connect).toHaveBeenCalledTimes(2);
  });

  test("does not reconnect when the server writes the same lock again", () => {
    const { provider, signal } = setup({ locked: true, at });

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
    const { metadata, provider } = setup({ locked: false, at });

    metadata.set("layoutDirection", "Horizontal");
    expect(provider.connect).not.toHaveBeenCalled();
  });

  test("stops when cleaned up", () => {
    const { provider, stop, signal } = setup({ locked: false, at });

    stop();
    signal(true, at + 1);
    expect(provider.connect).not.toHaveBeenCalled();
  });
});
