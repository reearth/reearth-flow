import {
  CaretDownIcon,
  CaretRightIcon,
  CopyIcon,
  MagnifyingGlassIcon,
} from "@phosphor-icons/react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { memo, ReactNode, useCallback, useMemo, useState } from "react";

import {
  Button,
  Dialog,
  DialogContent,
  DialogTitle,
  Input,
} from "@flow/components";
import { toast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import { resolveValue } from "@flow/utils/valueSummary";

type Props = {
  label: string;
  value: unknown;
  open: boolean;
  onClose: () => void;
};

const INDENT_PX = 14;

const ROW_HEIGHT = 22;

const MAX_PRIMITIVE_PREVIEW = 2000;

type NodeKind = "object" | "array" | "primitive";

type FlatNode = {
  id: string;
  depth: number;
  label: string | null;
  value: unknown;
  kind: NodeKind;
  childCount: number;
  expanded: boolean;
  closingBracket?: boolean;
};

function kindOf(value: unknown): NodeKind {
  if (Array.isArray(value)) return "array";
  if (value !== null && typeof value === "object") return "object";
  return "primitive";
}

function entriesOf(value: unknown, kind: NodeKind): [string, unknown][] {
  if (kind === "array") {
    return (value as unknown[]).map((v, i) => [String(i), v]);
  }
  if (kind === "object") {
    return Object.entries(value as Record<string, unknown>);
  }
  return [];
}

function flatten(root: unknown, expanded: Set<string>): FlatNode[] {
  const out: FlatNode[] = [];

  const walk = (
    rawValue: unknown,
    label: string | null,
    depth: number,
    id: string,
  ) => {
    const value = resolveValue(rawValue);
    const kind = kindOf(value);
    const entries = entriesOf(value, kind);
    const isExpanded = expanded.has(id);

    out.push({
      id,
      depth,
      label,
      value,
      kind,
      childCount: entries.length,
      expanded: isExpanded,
    });

    if (kind !== "primitive" && isExpanded) {
      for (const [childLabel, childValue] of entries) {
        walk(childValue, childLabel, depth + 1, `${id}.${childLabel}`);
      }
      out.push({
        id: `${id}:close`,
        depth,
        label: null,
        value,
        kind,
        childCount: entries.length,
        expanded: true,
        closingBracket: true,
      });
    }
  };

  walk(root, null, 0, "$");
  return out;
}

/** A count, grouped for the active language — `1,234` and `1.234` differ. */
function num(value: number): string {
  return value.toLocaleString(i18n.language);
}

function formatPrimitive(value: unknown): string {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  const str = typeof value === "string" ? value : String(value);
  if (str.length > MAX_PRIMITIVE_PREVIEW) {
    // The unit carries the meaning here — a bare number after an ellipsis
    // could be anything — so unlike the collapsed-node count below, this one
    // is a phrase and gets translated.
    return `${str.slice(0, MAX_PRIMITIVE_PREVIEW)}… ${i18n.t("({{n}} chars)", {
      n: num(str.length),
    })}`;
  }
  return str;
}

/**
 * What a collapsed container shows: its bracket and how many children it has.
 *
 * The words this used to carry — `3 items`, `5 keys` — said nothing the
 * bracket does not, and both needed an English plural rule. A glyph and a
 * count read the same in every language.
 */
function summarize(node: FlatNode): string {
  const bracket = node.kind === "array" ? "[]" : "{}";
  return `${bracket} ${num(node.childCount)}`;
}

function filterTree(root: unknown, query: string): FlatNode[] {
  const out: FlatNode[] = [];

  const walk = (
    rawValue: unknown,
    label: string | null,
    depth: number,
    id: string,
  ): boolean => {
    const value = resolveValue(rawValue);
    const kind = kindOf(value);
    const labelMatch = label !== null && label.toLowerCase().includes(query);

    if (kind === "primitive") {
      if (labelMatch || formatPrimitive(value).toLowerCase().includes(query)) {
        out.push({
          id,
          depth,
          label,
          value,
          kind,
          childCount: 0,
          expanded: false,
        });
        return true;
      }
      return false;
    }

    const entries = entriesOf(value, kind);
    const openIndex = out.length;
    out.push({
      id,
      depth,
      label,
      value,
      kind,
      childCount: entries.length,
      expanded: true,
    });

    let childMatched = false;
    for (const [childLabel, childValue] of entries) {
      if (walk(childValue, childLabel, depth + 1, `${id}.${childLabel}`)) {
        childMatched = true;
      }
    }

    if (childMatched || labelMatch) {
      out.push({
        id: `${id}:close`,
        depth,
        label: null,
        value,
        kind,
        childCount: entries.length,
        expanded: true,
        closingBracket: true,
      });
      return true;
    }

    out.length = openIndex;
    return false;
  };

  walk(root, null, 0, "$");
  return out;
}

function highlight(text: string, query: string): ReactNode {
  if (!query) return text;
  const lower = text.toLowerCase();
  const parts: ReactNode[] = [];
  let i = 0;
  while (i < text.length) {
    const idx = lower.indexOf(query, i);
    if (idx === -1) {
      parts.push(text.slice(i));
      break;
    }
    if (idx > i) parts.push(text.slice(i, idx));
    parts.push(
      <mark key={idx} className="rounded-sm bg-yellow-400/40 text-foreground">
        {text.slice(idx, idx + query.length)}
      </mark>,
    );
    i = idx + query.length;
  }
  return parts;
}

const RawJsonViewer: React.FC<Props> = ({ label, value, open, onClose }) => {
  const t = useT();

  const [scrollEl, setScrollEl] = useState<HTMLDivElement | null>(null);

  const [expanded, setExpanded] = useState<Set<string>>(() => new Set(["$"]));
  const [copied, setCopied] = useState(false);
  const [query, setQuery] = useState("");

  const trimmedQuery = query.trim().toLowerCase();
  const searching = trimmedQuery.length > 0;

  const nodes = useMemo(() => {
    if (!open) return [];
    return searching
      ? filterTree(value, trimmedQuery)
      : flatten(value, expanded);
  }, [open, value, expanded, searching, trimmedQuery]);

  const toggle = useCallback((id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const virtualizer = useVirtualizer({
    count: nodes.length,
    getScrollElement: () => scrollEl,
    estimateSize: () => ROW_HEIGHT,
    overscan: 20,
  });

  const handleCopy = useCallback(async () => {
    let json: string;
    try {
      json = JSON.stringify(resolveValue(value), null, 2);
    } catch {
      json = String(value);
    }
    try {
      await navigator.clipboard.writeText(json);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
      toast({
        title: t("JSON copied to clipboard"),
        description: t("JSON has been successfully copied to the clipboard."),
      });
    } catch {
      toast({
        title: t("Failed to copy JSON"),
        description: t("Unable to copy JSON to clipboard."),
        variant: "destructive",
      });
    }
  }, [value, t]);

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent size="3xl" className="h-[80vh]">
        <DialogTitle className="flex items-center justify-between gap-2 pr-12">
          <span className="truncate text-base">{label}</span>
          <Button
            variant="outline"
            size="sm"
            type="button"
            className="flex shrink-0 items-center gap-1 text-xs"
            onClick={handleCopy}>
            <CopyIcon size={12} />
            {copied ? t("Copied") : t("Copy JSON")}
          </Button>
        </DialogTitle>
        <div className="mx-4 flex shrink-0 items-center gap-2 rounded-md border border-border px-2">
          <MagnifyingGlassIcon
            size={14}
            className="shrink-0 text-muted-foreground"
          />
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t("Search keys and values") + "..."}
            className="h-8 border-0 bg-transparent px-0 focus-visible:ring-0"
          />
        </div>
        <div
          ref={setScrollEl}
          className="mx-4 mb-4 min-h-0 flex-1 overflow-auto rounded-md bg-muted/30 p-2 font-mono text-xs">
          {searching && nodes.length === 0 ? (
            <p className="p-2 text-muted-foreground">{t("No matches")}</p>
          ) : (
            <div
              className="relative w-full"
              style={{ height: `${virtualizer.getTotalSize()}px` }}>
              {virtualizer.getVirtualItems().map((virtualRow) => {
                const node = nodes[virtualRow.index];
                const expandable = node.kind !== "primitive";
                const closeBracket = node.kind === "array" ? "]" : "}";

                return (
                  <div
                    key={node.id}
                    className="absolute top-0 left-0 flex w-full items-center whitespace-nowrap"
                    style={{
                      height: `${ROW_HEIGHT}px`,
                      transform: `translateY(${virtualRow.start}px)`,
                      paddingLeft: `${node.depth * INDENT_PX}px`,
                    }}>
                    {node.closingBracket ? (
                      <span className="ml-5 text-muted-foreground/70">
                        {closeBracket}
                      </span>
                    ) : (
                      <>
                        {expandable && !searching ? (
                          <button
                            type="button"
                            className="mr-1 flex size-4 shrink-0 items-center justify-center text-muted-foreground hover:text-foreground"
                            onClick={() => toggle(node.id)}
                            aria-label={
                              node.expanded ? t("Collapse") : t("Expand")
                            }>
                            {node.expanded ? (
                              <CaretDownIcon size={12} />
                            ) : (
                              <CaretRightIcon size={12} />
                            )}
                          </button>
                        ) : expandable ? (
                          <span className="mr-1 flex size-4 shrink-0 items-center justify-center text-muted-foreground">
                            <CaretDownIcon size={12} />
                          </span>
                        ) : (
                          <span className="mr-1 size-4 shrink-0" />
                        )}

                        {node.label !== null && (
                          <span className="shrink-0 text-muted-foreground">
                            {highlight(node.label, trimmedQuery)}
                            {": "}
                          </span>
                        )}

                        {expandable ? (
                          <span className="text-muted-foreground/70">
                            {node.expanded
                              ? node.kind === "array"
                                ? "["
                                : "{"
                              : summarize(node)}
                          </span>
                        ) : (
                          <span className="truncate text-foreground">
                            {highlight(
                              formatPrimitive(node.value),
                              trimmedQuery,
                            )}
                          </span>
                        )}
                      </>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default memo(RawJsonViewer);
