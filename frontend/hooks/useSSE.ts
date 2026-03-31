"use client";

import { useEffect } from "react";
import { getSSEUrl } from "@/lib/api";
import { AlertEvent } from "@/lib/types";

export function useSSE(onPriceDrop: (event: AlertEvent) => void) {
  useEffect(() => {
    const es = new EventSource(getSSEUrl());

    es.addEventListener("price_drop", (e: MessageEvent) => {
      try {
        const event: AlertEvent = JSON.parse(e.data);
        onPriceDrop(event);
      } catch {
        console.error("Failed to parse SSE event", e.data);
      }
    });

    es.onerror = () => {
      console.warn("SSE connection error — browser will auto-reconnect");
    };

    return () => es.close();
  }, [onPriceDrop]);
}
