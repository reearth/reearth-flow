import { PerfectCursor } from "perfect-cursors";
import * as React from "react";

export function usePerfectCursor(
  cb: (point: number[]) => void,
  point?: number[],
) {
  const [pc] = React.useState(() => new PerfectCursor(cb));

  React.useLayoutEffect(() => {
    if (point) pc.addPoint(point);
    return () => pc.dispose();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pc]);

  const onPointChange = React.useCallback(
    (point: number[]) => pc.addPoint(point),
    [pc],
  );

  return onPointChange;
}
