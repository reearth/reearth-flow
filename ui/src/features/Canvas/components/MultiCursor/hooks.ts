import { PerfectCursor } from "perfect-cursors";
import { useCallback, useLayoutEffect, useState } from "react";

export function usePerfectCursor(
  cb: (point: number[]) => void,
  point?: number[],
) {
  const [pc] = useState(() => new PerfectCursor(cb));

  useLayoutEffect(() => {
    if (point) pc.addPoint(point);
    return () => pc.dispose();
  }, [point, pc]);

  const onPointChange = useCallback(
    (point: number[]) => pc.addPoint(point),
    [pc],
  );

  return onPointChange;
}
