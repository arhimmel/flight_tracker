"use client";

import { Alert } from "@/lib/types";

interface Props {
  alert: Alert;
  onDelete: (id: number) => void;
}

const statusStyles: Record<string, string> = {
  active: "bg-blue-100 text-blue-800",
  triggered: "bg-green-100 text-green-800",
  expired: "bg-gray-100 text-gray-500",
};

export function AlertCard({ alert, onDelete }: Props) {
  const priceDiff =
    alert.current_price != null
      ? alert.current_price - alert.target_price
      : null;

  return (
    <div className="rounded-xl border border-gray-200 bg-white p-4 shadow-sm flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <div>
          <span className="text-lg font-semibold">{alert.flight_number}</span>
          <span className="ml-2 text-sm text-gray-500">
            {alert.origin} → {alert.destination}
          </span>
        </div>
        <span
          className={`rounded-full px-2 py-0.5 text-xs font-medium ${statusStyles[alert.status] ?? statusStyles.active}`}
        >
          {alert.status}
        </span>
      </div>

      <div className="text-sm text-gray-600">
        Date: <span className="font-medium">{alert.flight_date}</span>
      </div>

      <div className="flex gap-4 text-sm">
        <div>
          Target:{" "}
          <span className="font-semibold text-gray-800">
            ${alert.target_price.toFixed(2)}
          </span>
        </div>
        {alert.current_price != null && (
          <div>
            Current:{" "}
            <span
              className={`font-semibold ${alert.current_price <= alert.target_price ? "text-green-600" : "text-gray-800"}`}
            >
              ${alert.current_price.toFixed(2)}
            </span>
            {priceDiff != null && (
              <span
                className={`ml-1 text-xs ${priceDiff <= 0 ? "text-green-500" : "text-red-400"}`}
              >
                ({priceDiff <= 0 ? "" : "+"}
                {priceDiff.toFixed(2)})
              </span>
            )}
          </div>
        )}
      </div>

      {alert.last_checked && (
        <div className="text-xs text-gray-400">
          Last checked: {new Date(alert.last_checked).toLocaleString()}
        </div>
      )}

      <button
        onClick={() => onDelete(alert.id)}
        className="mt-1 self-end rounded-lg border border-red-200 px-3 py-1 text-xs text-red-500 hover:bg-red-50 transition-colors"
      >
        Remove
      </button>
    </div>
  );
}
