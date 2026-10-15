import { useEffect, useState } from "react";

import { Button, LoadingSkeleton } from "@flow/components";
import { useLang, useT } from "@flow/lib/i18n";

/** A render in progress, and how long it has been going. */
export const Rendering: React.FC<{ message: string; startedAt?: number }> = ({
  message,
  startedAt,
}) => {
  const t = useT();
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 text-muted-foreground">
      <LoadingSkeleton className="mb-2" />
      <p className="text-sm">
        {message}
        {startedAt !== undefined && (
          <>
            {" "}
            <Elapsed since={startedAt} />
          </>
        )}
      </p>
      <p className="text-xs">
        {t("This usually takes a few seconds, and can take up to two minutes.")}
      </p>
    </div>
  );
};

/** Minutes and seconds since `since`, ticking every second. */
const Elapsed: React.FC<{ since: number }> = ({ since }) => {
  const lang = useLang();
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, []);

  const seconds = Math.max(0, Math.floor((now - since) / 1000));
  const minutes = Math.floor(seconds / 60);
  const rest = (seconds % 60).toLocaleString(lang, { minimumIntegerDigits: 2 });
  return (
    <span className="tabular-nums">
      {minutes.toLocaleString(lang)}:{rest}
    </span>
  );
};

/** Why there is nothing to show, and a way to ask again where that can help. */
export const Notice: React.FC<{
  message: string;
  detail?: string;
  onRetry?: () => void;
}> = ({ message, detail, onRetry }) => {
  const t = useT();
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 p-4 text-center">
      <p className="text-sm">{message}</p>
      {detail && <p className="text-xs text-muted-foreground">{detail}</p>}
      {onRetry && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          {t("Try again")}
        </Button>
      )}
    </div>
  );
};
