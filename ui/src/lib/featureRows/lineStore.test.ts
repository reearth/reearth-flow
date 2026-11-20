import { describe, expect, test } from "vitest";

import { DEFAULT_CHUNK_BYTES, LineStore, RowFilter } from "./lineStore";

const encoder = new TextEncoder();

const line = (id: string, attributes: Record<string, unknown> = {}) =>
  JSON.stringify({ id, attributes, geometry: "None" });

function storeOf(text: string, pieces = 1, options = {}) {
  const store = new LineStore(options);
  const bytes = encoder.encode(text);
  const size = Math.ceil(bytes.length / pieces);
  for (let offset = 0; offset < bytes.length; offset += size) {
    store.append(bytes.subarray(offset, offset + size));
  }
  store.finish();
  return store;
}

const ids = (store: LineStore) =>
  Array.from({ length: store.rowCount }, (_, row) => store.tableRow(row)?.id);

describe("LineStore", () => {
  test("indexes every line, wherever the chunks split them", () => {
    const text = ["a", "b", "c", "d"].map((id) => line(id)).join("\n");
    for (const pieces of [1, 3, 7, text.length]) {
      expect(ids(storeOf(text, pieces))).toEqual(["a", "b", "c", "d"]);
    }
  });

  test("keeps a multi-byte character split across chunks intact", () => {
    const text = line("渋谷", { name: "新宿駅" }) + "\n";
    const store = storeOf(text, text.length);
    expect(store.tableRow(0)).toMatchObject({
      id: "渋谷",
      attributes: { name: "新宿駅" },
    });
  });

  test("handles CRLF, blank lines, a missing final newline and bad lines", () => {
    const text = `${line("a")}\r\n\r\n   \n{not json\n${line("b")}`;
    const store = storeOf(text, 5);
    expect(ids(store)).toEqual(["a", "b"]);
    expect(store.skippedLines).toBe(1);
    expect(store.totalRows).toBe(2);
  });

  test("columns are the union of attribute names, in first-seen order", () => {
    const store = storeOf(
      [line("a", { x: 1 }), line("b", { y: 2, x: 3 }), line("c")].join("\n"),
    );
    expect(store.columns).toEqual(["x", "y"]);
  });

  test("rolls lines into new chunks without losing any", () => {
    const text = Array.from({ length: 50 }, (_, i) => line(`row-${i}`)).join(
      "\n",
    );
    const store = storeOf(text, 13, { chunkBytes: 64 });
    expect(ids(store)).toEqual(
      Array.from({ length: 50 }, (_, i) => `row-${i}`),
    );
  });

  test("keeps a line longer than a chunk", () => {
    const long = line("long", { text: "x".repeat(500) });
    const store = storeOf(`${line("a")}\n${long}\n${line("b")}`, 9, {
      chunkBytes: 32,
    });
    expect(ids(store)).toEqual(["a", "long", "b"]);
  });

  test("a line longer than a default chunk keeps later rows", () => {
    const long = line("long", { text: "x".repeat(DEFAULT_CHUNK_BYTES + 1024) });
    const text = [line("a"), long, line("b"), line("c")].join("\n");
    const store = storeOf(text, Math.ceil(text.length / (64 * 1024)));
    expect(store.truncated).toBe(false);
    expect(ids(store)).toEqual(["a", "long", "b", "c"]);
    expect(store.bytes).toBeLessThan(DEFAULT_CHUNK_BYTES * 4);
  });

  test("a line growing across many appends stays within the budget", () => {
    const long = line("long", { text: "x".repeat(4000) });
    const text = [line("a"), long, line("b")].join("\n");
    const store = storeOf(text, text.length / 10, {
      chunkBytes: 64,
      byteBudget: 64 * 1024,
    });
    expect(store.truncated).toBe(false);
    expect(ids(store)).toEqual(["a", "long", "b"]);
  });

  test("a row kept in one append is kept when it arrives in two", () => {
    const row = line("a", { t: "x".repeat(50) });
    const bytes = encoder.encode(row + "\n");
    const keeps = (byteBudget: number, pieces: Uint8Array[]) => {
      const store = new LineStore({ chunkBytes: 64, byteBudget });
      pieces.forEach((piece) => store.append(piece));
      store.finish();
      return store.rowCount === 1;
    };
    for (let budget = bytes.length; budget < 400; budget++) {
      if (!keeps(budget, [bytes])) continue;
      expect(
        keeps(budget, [bytes.subarray(0, 64), bytes.subarray(64)]),
        `budget ${budget}`,
      ).toBe(true);
    }
  });

  test("a small file does not take a whole default chunk", () => {
    const store = storeOf(line("a"));
    expect(store.bytes).toBeLessThan(DEFAULT_CHUNK_BYTES);
  });

  test("past the budget, stops keeping rows but still counts them", () => {
    const text = Array.from({ length: 100 }, (_, i) => line(`row-${i}`)).join(
      "\n",
    );
    const store = storeOf(text, 20, { chunkBytes: 256, byteBudget: 1024 });
    expect(store.truncated).toBe(true);
    expect(store.rowCount).toBeGreaterThan(0);
    expect(store.rowCount).toBeLessThan(100);
    expect(store.totalRows).toBe(100);
    expect(ids(store)).toEqual(
      Array.from({ length: store.rowCount }, (_, i) => `row-${i}`),
    );
  });

  test("past the sample, reads columns from the attributes alone", () => {
    const sample = Array.from({ length: 10 }, (_, i) => line(`s${i}`));
    const rest = [
      JSON.stringify({
        id: "a",
        attributes: { 'q"}]{[,': 'x\\"}', n: null },
        geometry: { attributes: { hidden: 1 } },
      }),
      JSON.stringify({
        id: "b",
        geometry: { attributes: { hidden: 2 } },
        attributes: { after: 1 },
      }),
      '{"id":"c","attributes":{"broken":}, "geometry":null}',
      '{"id":"d","attributes":null,"geometry":null}',
      '{"id":"e","attributes":{"cut":1},"geometry":[1,',
    ];
    const store = storeOf([...sample, ...rest].join("\n"));
    expect(store.columns).toEqual(['q"}]{[,', "n", "after"]);
    expect(ids(store).slice(10)).toEqual(["a", "b", "d"]);
    expect(store.skippedLines).toBe(2);
    expect(store.sample).toHaveLength(10);
  });

  test("past the sample, a line must be valid JSON to be a row", () => {
    const sample = Array.from({ length: 10 }, (_, i) => line(`s${i}`));
    const store = storeOf(
      [
        ...sample,
        '{"id":,"attributes":{"name":"A"}}',
        '{"\\u0061ttributes":{"escaped":1},"id":"ok"}',
      ].join("\n"),
    );
    expect(ids(store).slice(10)).toEqual(["ok"]);
    expect(store.columns).toEqual(["escaped"]);
    expect(store.skippedLines).toBe(1);
  });

  test("keeps the rows of a delivery that fit before dropping the rest", () => {
    const rows = [
      line("a", { t: "x".repeat(30) }),
      line("b", { t: "y".repeat(30) }),
    ];
    const text = rows.join("\n") + "\n";
    const storeWith = (byteBudget: number, pieces: string[]) => {
      const store = new LineStore({ byteBudget });
      pieces.forEach((piece) => store.append(encoder.encode(piece)));
      store.finish();
      return store;
    };
    const rowBytes = rows[0].length;
    const budget = rowBytes + 1 + 12 + rowBytes;
    for (const pieces of [[text], [rows[0] + "\n", rows[1] + "\n"]]) {
      const store = storeWith(budget, pieces);
      expect(store.truncated).toBe(true);
      expect(ids(store)).toEqual(["a"]);
      expect(store.totalRows).toBe(2);
    }
  });

  test("after trimming a chunk, later rows that fit are still kept", () => {
    const rows = Array.from({ length: 10 }, (_, i) => line(`r${i}`));
    const text = rows.join("\n") + "\n";
    const indexBytes = 16 * 12;
    const budget = text.length + indexBytes + text.length;
    const store = storeOf(text, 1, { byteBudget: budget });
    expect(store.truncated).toBe(false);
    expect(ids(store)).toEqual(rows.map((_, i) => `r${i}`));
  });

  test("a byte order mark does not hide the row", () => {
    const rows = Array.from({ length: 12 }, (_, i) => line(`r${i}`, { n: i }));
    rows[0] = "\uFEFF" + rows[0];
    rows[11] = "\uFEFF" + rows[11];
    const store = storeOf(rows.join("\n"));
    expect(store.tableRow(0)).toEqual({ id: "r0", attributes: { n: 0 } });
    expect(store.tableRow(11)).toEqual({ id: "r11", attributes: { n: 11 } });
    expect(store.skippedLines).toBe(0);
    const filter = new RowFilter("r11");
    filter.advance(store, () => false);
    expect(filter.count).toBe(1);
  });

  test("the row index counts against the budget", () => {
    const text = "{}\n".repeat(5000);
    const store = storeOf(text, 10, { byteBudget: text.length * 2 });
    expect(store.truncated).toBe(true);
    expect(store.bytes).toBeLessThanOrEqual(text.length * 2);
    expect(store.totalRows).toBe(5000);
  });

  test("table rows carry the id and attributes", () => {
    const store = storeOf(
      Array.from({ length: 12 }, (_, i) => line(`r${i}`, { n: i })).join("\n"),
    );
    expect(store.tableRow(11)).toEqual({ id: "r11", attributes: { n: 11 } });
    expect(store.tableRow(12)).toBeNull();
  });

  test("keeps the parsed sample for geometry detection", () => {
    const text = Array.from({ length: 20 }, (_, i) => line(`${i}`)).join("\n");
    expect(storeOf(text).sample).toHaveLength(10);
  });
});

describe("RowFilter", () => {
  const store = storeOf(
    [
      JSON.stringify({ id: "a", attributes: { city: "Tokyo" } }),
      JSON.stringify({
        id: "b",
        attributes: { city: "Osaka" },
        geometry: { note: "tokyo only in geometry" },
      }),
      JSON.stringify({ id: "tokyo-3", attributes: { city: "Nagoya" } }),
      JSON.stringify({ id: "d", attributes: { tags: ["TOKYO", "x"] } }),
    ].join("\n"),
  );

  const matchesOf = (term: string) => {
    const filter = new RowFilter(term);
    filter.advance(store, () => false);
    return Array.from(filter.matches.subarray(0, filter.count));
  };

  test("matches ids and attribute values, case-insensitively", () => {
    expect(matchesOf("tokyo")).toEqual([0, 2, 3]);
  });

  test("matches values that differ from the raw line's text", () => {
    const raw = storeOf(
      [
        '{"attributes":{"a":1}}',
        '{"id":"x","attributes":{"name":"caf\\u00e9"}}',
        '{"id":"y","attributes":{"size":1e3}}',
      ].join("\n"),
    );
    const matches = (term: string) => {
      const filter = new RowFilter(term);
      filter.advance(raw, () => false);
      return Array.from(filter.matches.subarray(0, filter.count));
    };
    expect(matches("0")).toContain(0);
    expect(matches("café")).toEqual([1]);
    expect(matches("1000")).toEqual([2]);
  });

  test("a term with no digit uses the raw-text check", () => {
    expect(matchesOf("-")).toEqual([2]);
    expect(matchesOf("k")).toEqual([0, 1, 2, 3]);
  });

  test("never matches on geometry alone", () => {
    expect(matchesOf("geometry")).toEqual([]);
  });

  test("resumes where it yielded, and maps rows back to view positions", () => {
    const filter = new RowFilter("tokyo");
    while (!filter.caughtUp(store)) filter.advance(store, () => true);
    expect(filter.count).toBe(3);
    expect(filter.rowAt(1)).toBe(2);
    expect(filter.indexOf(3)).toBe(2);
    expect(filter.indexOf(1)).toBe(-1);
  });

  test("filters rows that arrive after it started", () => {
    const growing = new LineStore();
    growing.append(encoder.encode(line("a", { v: "hit" }) + "\n"));
    const filter = new RowFilter("hit");
    filter.advance(growing, () => false);
    growing.append(encoder.encode(line("b", { v: "hit" }) + "\n"));
    growing.finish();
    filter.advance(growing, () => false);
    expect(filter.count).toBe(2);
  });
});
