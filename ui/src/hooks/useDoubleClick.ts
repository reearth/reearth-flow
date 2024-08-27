import { useCallback, useRef } from "react";

export default <TClick, TDoubleClick>(
  onClick: ((param?: TClick) => void) | undefined,
  onDoubleClick: ((param?: TDoubleClick) => void) | undefined,
  delay = 200,
): [(param?: TClick) => void, (param?: TDoubleClick) => void] => {
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  const singleClickHandler = useCallback(
    (param?: TClick) => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      } else if (onClick) {
        timerRef.current = setTimeout(() => {
          onClick(param);
          timerRef.current = null;
        }, delay);
      }
    },
    [onClick, delay],
  );

  const doubleClickHandler = useCallback(
    (param?: TDoubleClick) => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
      onDoubleClick?.(param);
    },
    [onDoubleClick],
  );

  return [singleClickHandler, doubleClickHandler];
};
