"use client";

import { useCallback, useEffect, useState } from "react";
import { fetchAlerts, deleteAlert } from "@/lib/api";
import { Alert } from "@/lib/types";

export function useAlerts(enabled = true) {
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      setLoading(true);
      const data = await fetchAlerts();
      setAlerts(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Unknown error");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (enabled) load();
  }, [load, enabled]);

  const remove = useCallback(
    async (id: number) => {
      await deleteAlert(id);
      setAlerts((prev) => prev.filter((a) => a.id !== id));
    },
    []
  );

  return { alerts, loading, error, refresh: load, remove };
}
