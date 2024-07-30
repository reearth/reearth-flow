import { equalityDeep } from "lib0/function";
import { useRef, useSyncExternalStore } from "react";
import * as Y from "yjs";

import { YJsonValue } from "./types";

type YTypeToJson<YType> =
  YType extends Y.Array<infer Value>
  ? Array<YTypeToJson<Value>>
  : YType extends Y.Map<infer MapValue>
  ? { [key: string]: YTypeToJson<MapValue> }
  : YType extends Y.XmlFragment | Y.XmlText
  ? string
  : YType;

export function useY<YType extends Y.AbstractType<any>>(yData: YType): YTypeToJson<YType> {
  const prevDataRef = useRef<YJsonValue | null>(null);
  return useSyncExternalStore(
    callback => {
      yData.observeDeep(callback);
      return () => yData.unobserveDeep(callback);
    },
    // Note: React requires reference equality
    () => {
      const data = yData.toJSON();
      if (equalityDeep(prevDataRef.current, data)) {
        return prevDataRef.current;
      } else {
        prevDataRef.current = data;
        return prevDataRef.current;
      }
    },
    () => yData.toJSON(),
  );
}
