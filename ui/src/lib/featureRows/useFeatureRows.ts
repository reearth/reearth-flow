import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";

import { safeSerialize } from "@flow/utils/valueSummary";

import { featureRowsClient } from "./client";
import type { FeatureRowsClient } from "./client";
import type { FeatureRowRecord, FeatureRowsStatus } from "./protocol";

export const ROW_BLOCK_SIZE = 100;
const MAX_CACHED_BLOCKS = 60;

export type FeatureTableRow = {
  _row: number;
  id: string;
  attributes: Record<string, unknown>;
  cells: Map<string, string>;
};

export function toTableRow(record: FeatureRowRecord): FeatureTableRow {
  return {
    _row: record.row,
    id: JSON.stringify(record.id || record.row),
    attributes: record.attributes,
    cells: new Map(),
  };
}

export function cellValue(row: FeatureTableRow, column: string): string {
  let value = row.cells.get(column);
  if (value === undefined) {
    value = safeSerialize(row.attributes[column] ?? null);
    row.cells.set(column, value);
  }
  return value;
}

type Block = { rows: FeatureTableRow[] };

const EMPTY_BLOCK: Block = { rows: [] };

const keyOf = (url: string, searchTerm: string, block: number) =>
  `${url}\u0000${searchTerm}\u0000${block}`;

export function useFeatureRows(
  url: string,
  client: FeatureRowsClient = featureRowsClient(),
) {
  const subscribe = useCallback(
    (onChange: () => void) => (url ? client.subscribe(url, onChange) : noop),
    [client, url],
  );
  const getSnapshot = useCallback(
    () => (url ? (client.status(url) ?? null) : null),
    [client, url],
  );
  const status = useSyncExternalStore(subscribe, getSnapshot);

  useEffect(() => {
    if (url) client.open(url);
  }, [client, url]);

  const search = useCallback(
    (term: string) => {
      if (url) client.search(url, term);
    },
    [client, url],
  );

  const [blocks, setBlocks] = useState<ReadonlyMap<string, Block>>(
    () => new Map(),
  );
  const inflight = useRef(new Map<string, Promise<Block>>());

  const searchTerm = status?.searchTerm ?? "";
  const viewCount = status?.viewCount ?? 0;

  const blockKey = useCallback(
    (block: number) => keyOf(url, searchTerm, block),
    [searchTerm, url],
  );

  const fetchBlock = useCallback(
    (block: number): Promise<Block> => {
      const key = blockKey(block);
      const existing = inflight.current.get(key);
      if (existing) return existing;
      const request = client
        .rows(url, block * ROW_BLOCK_SIZE, ROW_BLOCK_SIZE)
        .then((reply) => {
          const entry = { rows: reply.rows.map(toTableRow) };
          const answeredKey = keyOf(url, reply.searchTerm, block);
          setBlocks((previous) => {
            const next = new Map(previous);
            next.delete(answeredKey);
            next.set(answeredKey, entry);
            for (const oldest of next.keys()) {
              if (next.size <= MAX_CACHED_BLOCKS) break;
              next.delete(oldest);
            }
            return next;
          });
          return answeredKey === key ? entry : EMPTY_BLOCK;
        })
        .catch(() => EMPTY_BLOCK)
        .finally(() => inflight.current.delete(key));
      inflight.current.set(key, request);
      return request;
    },
    [blockKey, client, url],
  );

  const ensureRange = useCallback(
    (start: number, end: number) => {
      if (!url || viewCount === 0) return;
      const last = Math.min(end, viewCount - 1);
      for (
        let block = Math.floor(Math.max(0, start) / ROW_BLOCK_SIZE);
        block <= Math.floor(last / ROW_BLOCK_SIZE);
        block++
      ) {
        const key = blockKey(block);
        const cached = blocks.get(key);
        const expected = Math.min(
          ROW_BLOCK_SIZE,
          viewCount - block * ROW_BLOCK_SIZE,
        );
        if (
          (!cached || cached.rows.length < expected) &&
          !inflight.current.has(key)
        ) {
          void fetchBlock(block);
        }
      }
    },
    [blockKey, blocks, fetchBlock, url, viewCount],
  );

  const getRow = useCallback(
    (index: number): FeatureTableRow | undefined =>
      blocks.get(blockKey(Math.floor(index / ROW_BLOCK_SIZE)))?.rows[
        index % ROW_BLOCK_SIZE
      ],
    [blockKey, blocks],
  );

  const ensureRow = useCallback(
    async (index: number) => {
      const existing = getRow(index);
      if (existing) return existing;
      const fetched = await fetchBlock(Math.floor(index / ROW_BLOCK_SIZE));
      return fetched.rows[index % ROW_BLOCK_SIZE];
    },
    [fetchBlock, getRow],
  );

  const readLine = useCallback(
    (row: number) =>
      url ? client.line(url, row).catch(() => null) : Promise.resolve(null),
    [client, url],
  );

  const viewIndexOf = useCallback(
    (row: number) =>
      url ? client.viewIndexOf(url, row).catch(() => -1) : Promise.resolve(-1),
    [client, url],
  );

  return useMemo(
    () => ({
      status,
      search,
      getRow,
      ensureRange,
      ensureRow,
      readLine,
      viewIndexOf,
    }),
    [status, search, getRow, ensureRange, ensureRow, readLine, viewIndexOf],
  );
}

export type FeatureRows = ReturnType<typeof useFeatureRows>;

export type { FeatureRowsStatus };

function noop() {}
