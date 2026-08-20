import { LockIcon, LockOpenIcon } from "@phosphor-icons/react";
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

import { useT } from "@flow/lib/i18n";
import { cn } from "@flow/lib/utils";

type Props = {
  onUnlock: () => void;
};

const UNLOCK_HOVER_DELAY = 1000;

const LockedBadge: React.FC<Props> = ({ onUnlock }) => {
  const t = useT();
  const [isUnlockReady, setIsUnlockReady] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const lockedLabelRef = useRef<HTMLSpanElement>(null);
  const unlockLabelRef = useRef<HTMLSpanElement>(null);
  const [labelWidth, setLabelWidth] = useState<number>();

  useLayoutEffect(() => {
    const activeLabel = isUnlockReady
      ? unlockLabelRef.current
      : lockedLabelRef.current;
    if (activeLabel) setLabelWidth(activeLabel.offsetWidth);
  }, [isUnlockReady, t]);

  const handleMouseEnter = useCallback(() => {
    timerRef.current = setTimeout(
      () => setIsUnlockReady(true),
      UNLOCK_HOVER_DELAY,
    );
  }, []);

  const handleMouseLeave = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    setIsUnlockReady(false);
  }, []);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  return (
    <div
      className={cn(
        "flex max-w-fit items-center gap-2 rounded-xl bg-accent/50 px-3 py-2 text-xs transition-colors",
        isUnlockReady ? "cursor-pointer" : "cursor-default",
      )}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      onDoubleClick={isUnlockReady ? onUnlock : undefined}>
      <div className="relative size-[18px]">
        <LockIcon
          weight="thin"
          size={18}
          className={cn(
            "absolute inset-0 transition-all duration-300",
            isUnlockReady ? "opacity-0" : "scale-100 opacity-100",
          )}
        />
        <LockOpenIcon
          weight="thin"
          size={18}
          className={cn(
            "absolute inset-0 transition-all duration-200",
            isUnlockReady ? "scale-100 opacity-100" : "opacity-0",
          )}
        />
      </div>
      {/* Both labels share one grid cell (crossfade); the container width is
          driven to the active label so the badge resizes as it transitions. */}
      <div
        className="grid overflow-hidden font-light whitespace-nowrap text-accent-foreground transition-[width] duration-300 select-none"
        style={{ width: labelWidth }}>
        <span
          ref={lockedLabelRef}
          className={cn(
            "col-start-1 row-start-1 w-fit justify-self-start transition-opacity duration-300",
            isUnlockReady ? "opacity-0" : "opacity-100",
          )}>
          {t("Locked")}
        </span>
        <span
          ref={unlockLabelRef}
          className={cn(
            "col-start-1 row-start-1 w-fit justify-self-start transition-opacity duration-300",
            isUnlockReady ? "opacity-100" : "opacity-0",
          )}>
          {t("Unlock")}
        </span>
      </div>
    </div>
  );
};

export default LockedBadge;
