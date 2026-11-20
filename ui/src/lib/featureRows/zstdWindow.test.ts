import { describe, expect, test } from "vitest";

import { concatBytes, rawZstdFrame } from "./zstdTestFrame";
import { ZstdWindowGuard } from "./zstdWindow";

const fixture = () =>
  rawZstdFrame(new TextEncoder().encode('{"id":1}\n'.repeat(20000)));

const pushAll = (guard: ZstdWindowGuard, bytes: Uint8Array, size: number) => {
  for (let offset = 0; offset < bytes.length; offset += size) {
    guard.push(bytes.subarray(offset, offset + size));
  }
};

describe("ZstdWindowGuard", () => {
  test("accepts a real stream, however it is split", () => {
    for (const size of [1, 3, 64, 4096]) {
      expect(() =>
        pushAll(new ZstdWindowGuard(), fixture(), size),
      ).not.toThrow();
    }
  });

  test("checks every frame, past skippable frames", () => {
    const skippable = new Uint8Array([
      0x50, 0x2a, 0x4d, 0x18, 2, 0, 0, 0, 9, 9,
    ]);
    const big = new Uint8Array([0x28, 0xb5, 0x2f, 0xfd, 0x00, 0xa0]);
    const stream = concatBytes([fixture(), skippable, big]);
    expect(() => pushAll(new ZstdWindowGuard(), stream, 7)).toThrow(/window/);
  });

  test("refuses a window descriptor over the limit", () => {
    const header = new Uint8Array([0x28, 0xb5, 0x2f, 0xfd, 0x00, 0xa0]);
    expect(() => new ZstdWindowGuard().push(header)).toThrow(/window/);
  });

  test("refuses a single-segment frame whose content is over the limit", () => {
    const header = new Uint8Array([
      0x28, 0xb5, 0x2f, 0xfd, 0xa0, 0x00, 0x00, 0x00, 0x40,
    ]);
    expect(() => new ZstdWindowGuard().push(header)).toThrow(/window/);
    expect(() => new ZstdWindowGuard(2 ** 31).push(header)).not.toThrow();
  });

  test("refuses data that is not Zstandard", () => {
    expect(() =>
      new ZstdWindowGuard().push(new TextEncoder().encode('{"id":1}')),
    ).toThrow(/Zstandard/);
  });
});
