"use client";

import { useCallback } from "react";
import toast from "react-hot-toast";

import { AlertForm } from "@/components/AlertForm";
import { AlertList } from "@/components/AlertList";
import { useAlerts } from "@/hooks/useAlerts";
import { useSSE } from "@/hooks/useSSE";
import { Alert, AlertEvent } from "@/lib/types";

export default function Home() {
  const { alerts, loading, error, refresh, remove } = useAlerts();

  const handlePriceDrop = useCallback(
    (event: AlertEvent) => {
      toast.success(
        `Price dropped! ${event.flight_number} (${event.origin}→${event.destination}) on ${event.flight_date} is now $${event.current_price.toFixed(2)} — your target was $${event.target_price.toFixed(2)}`,
        { duration: 8000 }
      );
      refresh();
    },
    [refresh]
  );

  useSSE(handlePriceDrop);

  const handleCreated = useCallback(
    (alert: Alert) => {
      toast.success(`Alert added for ${alert.flight_number}`);
      refresh();
    },
    [refresh]
  );

  return (
    <div className="flex flex-col gap-8">
      <AlertForm onCreated={handleCreated} />
      <section>
        <h2 className="mb-4 text-base font-semibold text-gray-700">
          Active Alerts
        </h2>
        <AlertList
          alerts={alerts}
          loading={loading}
          error={error}
          onDelete={remove}
        />
      </section>
    </div>
  );
}
