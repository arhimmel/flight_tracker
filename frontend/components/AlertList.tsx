"use client";

import { Alert } from "@/lib/types";
import { AlertCard } from "./AlertCard";

interface Props {
  alerts: Alert[];
  loading: boolean;
  error: string | null;
  onDelete: (id: number) => void;
}

export function AlertList({ alerts, loading, error, onDelete }: Props) {
  if (loading) {
    return (
      <div className="text-center text-gray-400 py-12">Loading alerts…</div>
    );
  }

  if (error) {
    return (
      <div className="text-center text-red-400 py-12">Error: {error}</div>
    );
  }

  if (alerts.length === 0) {
    return (
      <div className="text-center text-gray-400 py-12">
        No alerts yet. Add one above!
      </div>
    );
  }

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {alerts.map((alert) => (
        <AlertCard key={alert.id} alert={alert} onDelete={onDelete} />
      ))}
    </div>
  );
}
