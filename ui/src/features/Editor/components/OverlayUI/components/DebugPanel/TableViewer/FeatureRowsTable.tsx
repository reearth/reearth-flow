import { CircleNotchIcon } from "@phosphor-icons/react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { KeyboardEvent } from "react";
import { memo, useCallback, useEffect, useRef, useState } from "react";

import {
  Button,
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  Input,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@flow/components";
import { cellValue } from "@flow/lib/featureRows";
import type { FeatureRows, FeatureTableRow } from "@flow/lib/featureRows";
import { useT } from "@flow/lib/i18n";

const ID_COLUMN = "\u0000id";
const ROW_HEIGHT = 24;
const COLUMN_WIDTH = 200;
const COLUMN_MAX_WIDTH = 400;
const DISPLAY_MAX_CHARS = 100;
const SEARCH_DEBOUNCE_MS = 250;

const truncateDisplayValue = (value: string) =>
  value.length <= DISPLAY_MAX_CHARS
    ? value
    : value.slice(0, DISPLAY_MAX_CHARS) + "…";

const columnStyle = { width: COLUMN_WIDTH, maxWidth: COLUMN_MAX_WIDTH };

type Props = {
  rows: FeatureRows;
  selectedRow: number | null;
  onRowClick?: (row: FeatureTableRow) => void;
  onRowDoubleClick?: (row: FeatureTableRow) => void;
};

const FeatureRowsTable: React.FC<Props> = ({
  rows,
  selectedRow,
  onRowClick,
  onRowDoubleClick,
}) => {
  const t = useT();
  const { status, search, getRow, ensureRange, ensureRow, viewIndexOf } = rows;
  const viewCount = status?.viewCount ?? 0;
  const searchTerm = status?.searchTerm ?? "";
  const columns = status?.columns;

  const [searchInput, setSearchInput] = useState("");
  useEffect(() => {
    const timer = setTimeout(
      () => search(searchInput),
      searchInput ? SEARCH_DEBOUNCE_MS : 0,
    );
    return () => clearTimeout(timer);
  }, [search, searchInput]);

  const [hiddenColumns, setHiddenColumns] = useState<ReadonlySet<string>>(
    () => new Set(),
  );
  const allColumns = [ID_COLUMN, ...(columns ?? [])];
  const visibleColumns = allColumns.filter((key) => !hiddenColumns.has(key));
  const columnLabel = (key: string) => (key === ID_COLUMN ? "Feature ID" : key);

  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: viewCount,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 20,
  });

  const items = virtualizer.getVirtualItems();
  const firstIndex = items[0]?.index ?? 0;
  const lastIndex = items[items.length - 1]?.index ?? -1;
  useEffect(() => {
    if (lastIndex >= firstIndex) ensureRange(firstIndex, lastIndex);
  }, [ensureRange, firstIndex, lastIndex]);

  const navigation = useRef(0);

  useEffect(() => {
    navigation.current++;
    if (parentRef.current) parentRef.current.scrollTop = 0;
  }, [searchTerm]);

  const [activeIndex, setActiveIndex] = useState(0);

  const reveal = useCallback(
    (index: number) => {
      setActiveIndex(index);
      const visible = virtualizer.getVirtualItems();
      const start = visible[0]?.index ?? 0;
      const end = visible[visible.length - 1]?.index ?? -1;
      if (index >= start && index <= end) return;
      virtualizer.scrollToIndex(index, { align: "start", behavior: "auto" });
    },
    [virtualizer],
  );

  const searching = status?.searching ?? false;
  const lookupKey = `${selectedRow}\u0000${searchTerm}`;
  const missedLookup = useRef<string | null>(null);

  useEffect(() => {
    missedLookup.current = null;
    if (selectedRow === null) return;
    let cancelled = false;
    void viewIndexOf(selectedRow).then((index) => {
      if (cancelled) return;
      if (index < 0) missedLookup.current = lookupKey;
      else reveal(index);
    });
    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedRow, searchTerm, viewIndexOf]);

  useEffect(() => {
    const key = lookupKey;
    if (selectedRow === null || missedLookup.current !== key) return;
    void viewIndexOf(selectedRow).then((index) => {
      if (index < 0 || missedLookup.current !== key) return;
      missedLookup.current = null;
      reveal(index);
    });
  }, [lookupKey, reveal, searching, selectedRow, viewCount, viewIndexOf]);

  const [parentHeight, setParentHeight] = useState(0);
  useEffect(() => {
    const element = parentRef.current;
    if (!element) return;
    const observer = new ResizeObserver(() =>
      setParentHeight(element.clientHeight),
    );
    observer.observe(element);
    setParentHeight(element.clientHeight);
    return () => observer.disconnect();
  }, []);

  const rowRefs = useRef(new Map<number, HTMLTableRowElement>());

  const moveActive = useCallback(
    (delta: number) => {
      if (!viewCount) return;
      const next = Math.max(0, Math.min(activeIndex + delta, viewCount - 1));
      if (next === activeIndex) return;
      const token = ++navigation.current;
      setActiveIndex(next);
      virtualizer.scrollToIndex(next, { align: "auto" });
      requestAnimationFrame(() => rowRefs.current.get(next)?.focus());
      void ensureRow(next).then((row) => {
        if (row && token === navigation.current) onRowClick?.(row);
      });
    },
    [activeIndex, ensureRow, onRowClick, viewCount, virtualizer],
  );

  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLTableRowElement>) => {
      if (event.key === "Enter") {
        event.preventDefault();
        const row = getRow(activeIndex);
        if (row) onRowDoubleClick?.(row);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        moveActive(-1);
      } else if (event.key === "ArrowDown") {
        event.preventDefault();
        moveActive(1);
      }
    },
    [activeIndex, getRow, moveActive, onRowDoubleClick],
  );

  const spacerHeight = Math.max(virtualizer.getTotalSize(), parentHeight);
  const tabStop =
    activeIndex >= firstIndex && activeIndex <= lastIndex
      ? activeIndex
      : firstIndex;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex items-center gap-4 py-1">
        <Input
          placeholder={t("Search") + "..."}
          value={searchInput}
          onChange={(event) => setSearchInput(event.target.value)}
          className="max-w-sm"
        />
        <DropdownMenu>
          <DropdownMenuTrigger
            render={
              <Button variant="outline" size="sm" className="ml-auto">
                {t("Columns")}
              </Button>
            }
          />
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => setHiddenColumns(new Set())}>
              {t("Select All")}
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => setHiddenColumns(new Set(allColumns))}>
              {t("Deselect All")}
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            {allColumns.map((key) => (
              <DropdownMenuCheckboxItem
                key={key}
                className="capitalize"
                checked={!hiddenColumns.has(key)}
                onCheckedChange={(checked) =>
                  setHiddenColumns((previous) => {
                    const next = new Set(previous);
                    if (checked) next.delete(key);
                    else next.add(key);
                    return next;
                  })
                }>
                {columnLabel(key)}
              </DropdownMenuCheckboxItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
      <div
        ref={parentRef}
        className="min-h-0 flex-1 overflow-auto rounded-md"
        style={{ contain: "paint", willChange: "transform" }}>
        <div className="relative w-full" style={{ height: spacerHeight }}>
          <Table className="w-full text-xs">
            <TableHeader>
              <TableRow>
                {visibleColumns.map((key) => (
                  <TableHead key={key} className="h-8" style={columnStyle}>
                    {columnLabel(key)}
                  </TableHead>
                ))}
              </TableRow>
            </TableHeader>
            <TableBody>
              {viewCount ? (
                items.map((item, position) => {
                  const row = getRow(item.index);
                  return (
                    <TableRow
                      key={item.key}
                      ref={(element) => {
                        if (element) rowRefs.current.set(item.index, element);
                        else rowRefs.current.delete(item.index);
                      }}
                      tabIndex={item.index === tabStop ? 0 : -1}
                      onKeyDown={handleKeyDown}
                      onFocus={() => {
                        if (item.index !== activeIndex) navigation.current++;
                        setActiveIndex(item.index);
                      }}
                      className="after:border-line-200 relative cursor-pointer border-0 after:absolute after:top-0 after:left-0 after:z-10 after:w-full after:border-b focus-visible:outline-hidden"
                      style={{
                        height: item.size,
                        transform: `translateY(${item.start - position * item.size}px)`,
                      }}
                      data-state={
                        row && row._row === selectedRow ? "selected" : undefined
                      }
                      onClick={() => {
                        navigation.current++;
                        setActiveIndex(item.index);
                        if (row) onRowClick?.(row);
                      }}
                      onDoubleClick={() => row && onRowDoubleClick?.(row)}>
                      {visibleColumns.map((key) => (
                        <TableCell
                          key={key}
                          className="px-2 py-[2px]"
                          style={columnStyle}>
                          {row ? (
                            key === ID_COLUMN ? (
                              row.id
                            ) : (
                              truncateDisplayValue(cellValue(row, key))
                            )
                          ) : (
                            <div className="h-3 w-2/3 animate-pulse rounded bg-muted" />
                          )}
                        </TableCell>
                      ))}
                    </TableRow>
                  );
                })
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={visibleColumns.length || 1}
                    className="h-24 text-center">
                    {status?.searching ? (
                      <CircleNotchIcon className="mx-auto size-4 animate-spin text-muted-foreground" />
                    ) : (
                      t("No Results")
                    )}
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>
      </div>
    </div>
  );
};

export default memo(FeatureRowsTable);
