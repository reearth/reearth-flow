import { useCallback, useEffect, useMemo, useRef } from "react";

import { config } from "@flow/config";

import type { EmbedInboundMessage, EmbedOutboundMessage } from "./messages";
import { parseInboundMessage } from "./messages";

export const getEmbedParentOrigin = (): string | undefined => {
  const dashboardUrl = config().reearthDashboardUrl;
  if (!dashboardUrl) return;
  try {
    return new URL(dashboardUrl).origin;
  } catch {
    return;
  }
};

export const isFramed = (): boolean => {
  try {
    return window.parent !== window;
  } catch {
    return true;
  }
};

export const useEmbedBridge = (
  onMessage?: (message: EmbedInboundMessage) => void,
) => {
  const parentOrigin = useMemo(
    () => (isFramed() ? getEmbedParentOrigin() : undefined),
    [],
  );

  const onMessageRef = useRef(onMessage);
  useEffect(() => {
    onMessageRef.current = onMessage;
  }, [onMessage]);

  useEffect(() => {
    if (!parentOrigin) return;

    const handleMessage = (event: MessageEvent) => {
      if (event.origin !== parentOrigin || event.source !== window.parent)
        return;
      const message = parseInboundMessage(event.data);
      if (message) onMessageRef.current?.(message);
    };

    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  }, [parentOrigin]);

  const post = useCallback(
    (message: EmbedOutboundMessage) => {
      if (!parentOrigin) return;
      window.parent.postMessage(message, parentOrigin);
    },
    [parentOrigin],
  );

  return { isConnected: !!parentOrigin, post };
};
