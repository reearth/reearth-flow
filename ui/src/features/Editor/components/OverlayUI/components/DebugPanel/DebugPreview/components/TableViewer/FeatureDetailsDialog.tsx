import { memo, useState } from "react";

import { Button } from "@flow/components";
import {
  Dialog,
  DialogContent,
  DialogContentWrapper,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@flow/components/Dialog";
import { useT } from "@flow/lib/i18n";

type Props = {
  feature: any | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

const FeatureDetailsDialog: React.FC<Props> = ({
  feature,
  open,
  onOpenChange,
}) => {
  const t = useT();
  const [copiedKey, setCopiedKey] = useState<string | null>(null);

  const handleCopy = async (key: string, value: any) => {
    try {
      // Try to get the original value first, fallback to the displayed value
      const originalKey = `${key}_original`;
      const originalValue = feature[originalKey];
      const valueToCopy = originalValue !== undefined ? originalValue : value;

      const textToCopy =
        typeof valueToCopy === "string"
          ? valueToCopy
          : JSON.stringify(valueToCopy, null, 2);
      await navigator.clipboard.writeText(textToCopy);
      setCopiedKey(key);
      setTimeout(() => setCopiedKey(null), 2000);
    } catch (error) {
      console.error("Failed to copy:", error);
    }
  };

  const renderValue = (value: any, _key: string) => {
    if (value === null || value === undefined) {
      return <span className="text-muted-foreground italic">null</span>;
    }

    if (typeof value === "string") {
      // Handle JSON strings (like attributes.attributes)
      if (value.startsWith("{") || value.startsWith("[")) {
        try {
          const parsed = JSON.parse(value);
          return (
            <div className="space-y-2">
              <div className="text-xs text-muted-foreground">
                {t("JSON String")} ({value.length.toLocaleString()}{" "}
                {t("characters")})
              </div>
              <pre className="max-h-96 overflow-y-auto rounded bg-muted p-3 font-mono text-xs break-words whitespace-pre-wrap">
                {JSON.stringify(parsed, null, 2)}
              </pre>
            </div>
          );
        } catch {
          // Not valid JSON, show as regular string
        }
      }

      return (
        <div className="space-y-1">
          {value.length > 100 && (
            <div className="text-xs text-muted-foreground">
              {value.length.toLocaleString()} {t("characters")}
            </div>
          )}
          <div className="break-words">{value}</div>
        </div>
      );
    }

    if (typeof value === "object") {
      return (
        <pre className="max-h-96 overflow-y-auto rounded bg-muted p-3 font-mono text-xs break-words whitespace-pre-wrap">
          {JSON.stringify(value, null, 2)}
        </pre>
      );
    }

    return <span>{String(value)}</span>;
  };

  if (!feature) return null;

  const entries = Object.entries(feature).filter(
    ([key]) => !key.endsWith("_original"),
  ); // Hide original values from display;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex flex-col" size="lg">
        <DialogHeader>
          <DialogTitle>{t("Feature Details")}</DialogTitle>
        </DialogHeader>
        <DialogContentWrapper className="h-[500px]">
          <div className="flex-1 space-y-2 overflow-auto pr-4">
            {entries.map(([key, value]) => (
              <div key={key} className="flex flex-col">
                <div className="flex justify-between space-y-1">
                  <div className="flex w-full items-center justify-between">
                    <h4 className="flex w-[175px] shrink-0 flex-col text-sm font-medium break-all">
                      <code className="rounded px-1 py-0.5 text-xs">{key}</code>
                      <span className="text-xs text-muted-foreground">
                        ({typeof value})
                      </span>
                    </h4>
                    <div className="wrap flex-1 pl-4 text-xs break-all">
                      {renderValue(value, key)}
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleCopy(key, value)}
                      className="h-6 shrink-0 px-2 text-xs">
                      {copiedKey === key ? t("Copied!") : t("Copy")}
                    </Button>
                  </div>
                </div>
                {entries.indexOf([key, value]) < entries.length - 1 && (
                  <div className="mt-4 border-t border-border" />
                )}
              </div>
            ))}
          </div>
        </DialogContentWrapper>
        <DialogFooter className="border-t pt-4">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("Close")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default memo(FeatureDetailsDialog);
