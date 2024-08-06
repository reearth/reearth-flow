import { useEffect, useRef } from "react";

export const useWebSocket = (url: string): {
  send: (v: string) => void,
} => {
  const urlObject = new URL(url);
  if (urlObject.protocol !== 'ws:') {
    throw TypeError('protocol is invalid');
  }

  const queue = useRef<string[]>([]);
  const wsRef = useRef<Map<string, WebSocket>>(new Map());

  useEffect(() => {
    wsRef.current.set(url, wsRef.current.get(url) || new WebSocket(url));
  })

  const send = (v: string) => {
    const ws = wsRef.current?.get(url);
    if (ws) {
      while (queue.current.length > 0) {
        const v0 = queue.current.shift();
        if (v0) {
          ws.send(v0);
        }
      }
      ws.send(v);
    } else {
      queue.current.push(v)
    }
  };

  return {
    send,
  }
}