import { debounce } from "lodash-es";
import { useEffect, useRef } from "react";

export function useDebouncedCallback<TArgs extends unknown[]>(
  callback: (...args: TArgs) => void,
  delay: number,
) {
  const callbackRef = useRef(callback);

  useEffect(() => {
    callbackRef.current = callback;
  }, [callback]);

  return useRef(
    debounce((...args: TArgs) => callbackRef.current(...args), delay),
  ).current;
}
