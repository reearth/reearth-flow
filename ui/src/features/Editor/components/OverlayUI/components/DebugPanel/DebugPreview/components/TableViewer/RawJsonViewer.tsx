import { CaretDownIcon, CaretRightIcon, CopyIcon } from "@phosphor-icons/react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { memo, useCallback, useMemo, useState } from "react";

import { Button, Dialog, DialogContent, DialogTitle } from "@flow/components";
import { useT } from "@flow/lib/i18n";

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

function resolveValue(value: unknown): unknown {
  if (typeof value === "string") {
    try {
      const parsed = JSON.parse(value);
      if (typeof parsed === "object" && parsed !== null) return parsed;
    } catch {
      console.log("Not valid JSON — keep the original string");
    }
  }
  return value;
}

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

function formatPrimitive(value: unknown): string {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  const str = typeof value === "string" ? value : String(value);
  if (str.length > MAX_PRIMITIVE_PREVIEW) {
    return `${str.slice(0, MAX_PRIMITIVE_PREVIEW)}… (${str.length.toLocaleString()} chars)`;
  }
  return str;
}

function summarize(node: FlatNode): string {
  if (node.kind === "array") return `[] ${node.childCount} items`;
  return `{} ${node.childCount} keys`;
}

const RawJsonViewer: React.FC<Props> = ({ label, value, open, onClose }) => {
  const t = useT();

  const [scrollEl, setScrollEl] = useState<HTMLDivElement | null>(null);

  const [expanded, setExpanded] = useState<Set<string>>(() => new Set(["$"]));
  const [copied, setCopied] = useState(false);

  const nodes = useMemo(
    () => (open ? flatten(value, expanded) : []),
    [open, value, expanded],
  );

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
    } catch {
      console.log("Failed to copy to clipboard");
    }
  }, [value]);

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
        <div
          ref={setScrollEl}
          className="mx-4 mb-4 min-h-0 flex-1 overflow-auto rounded-md bg-muted/30 p-2 font-mono text-xs">
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
                      {expandable ? (
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
                      ) : (
                        <span className="mr-1 size-4 shrink-0" />
                      )}

                      {node.label !== null && (
                        <span className="shrink-0 text-muted-foreground">
                          {node.label}
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
                          {formatPrimitive(node.value)}
                        </span>
                      )}
                    </>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};

export default memo(RawJsonViewer);
