import { useCallback, useRef } from "react";

export default (
  onClick: ((param?: any) => void) | undefined,
  onDoubleClick: ((param?: any) => void) | undefined,
  delay = 200
): [(param?: any) => void, (param?: any) => void] => {
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  const singleClickHandler = useCallback(
    (param?: any) => {
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
    [onClick, delay]
  );

  const doubleClickHandler = useCallback(
    (param?: any) => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
      onDoubleClick?.(param);
    },
    [onDoubleClick]
  );

  return [singleClickHandler, doubleClickHandler];
};
