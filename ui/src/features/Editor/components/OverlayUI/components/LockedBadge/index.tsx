import { LockIcon, LockOpenIcon } from "@phosphor-icons/react";
import { useCallback, useEffect, useRef, useState } from "react";

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
        "flex items-center gap-2 rounded-xl bg-accent/50 px-6 py-2 text-xs transition-colors",
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
      <div className="relative font-light whitespace-nowrap text-accent-foreground select-none">
        {/* invisible width-setter — holds the wider string so the container never grows */}
        <span className="invisible" aria-hidden="true">
          {t("Unlock")}
        </span>
        <span
          className={cn(
            "absolute inset-0 transition-opacity duration-300",
            isUnlockReady ? "opacity-0" : "opacity-100",
          )}>
          {t("Locked")}
        </span>
        <span
          className={cn(
            "absolute inset-0 transition-opacity duration-300",
            isUnlockReady ? "opacity-100" : "opacity-0",
          )}>
          {t("Unlock")}
        </span>
      </div>
    </div>
  );
};

export default LockedBadge;
