"use client";

import { useState } from "react";
import { createAlert } from "@/lib/api";
import { Alert, CreateAlertPayload } from "@/lib/types";

interface Props {
  onCreated: (alert: Alert) => void;
}

const empty: CreateAlertPayload = {
  flight_number: "",
  flight_date: "",
  origin: "",
  destination: "",
  target_price: 0,
};

export function AlertForm({ onCreated }: Props) {
  const [form, setForm] = useState<CreateAlertPayload>(empty);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const set = (field: keyof CreateAlertPayload) =>
    (e: React.ChangeEvent<HTMLInputElement>) =>
      setForm((prev) => ({
        ...prev,
        [field]: field === "target_price" ? parseFloat(e.target.value) : e.target.value,
      }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      const alert = await createAlert(form);
      onCreated(alert);
      setForm(empty);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Something went wrong");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="rounded-xl border border-gray-200 bg-white p-6 shadow-sm"
    >
      <h2 className="mb-4 text-lg font-semibold text-gray-800">
        Track a Flight
      </h2>

      <div className="grid gap-4 sm:grid-cols-2">
        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Flight Number
          </label>
          <input
            required
            placeholder="AA123"
            value={form.flight_number}
            onChange={set("flight_number")}
            className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-400"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Date
          </label>
          <input
            required
            type="date"
            value={form.flight_date}
            onChange={set("flight_date")}
            className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-400"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Origin (IATA)
          </label>
          <input
            required
            placeholder="JFK"
            maxLength={3}
            value={form.origin}
            onChange={set("origin")}
            className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm uppercase focus:outline-none focus:ring-2 focus:ring-blue-400"
          />
        </div>

        <div>
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Destination (IATA)
          </label>
          <input
            required
            placeholder="LAX"
            maxLength={3}
            value={form.destination}
            onChange={set("destination")}
            className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm uppercase focus:outline-none focus:ring-2 focus:ring-blue-400"
          />
        </div>

        <div className="sm:col-span-2">
          <label className="mb-1 block text-sm font-medium text-gray-700">
            Target Price (USD)
          </label>
          <div className="relative">
            <span className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
              $
            </span>
            <input
              required
              type="number"
              min="1"
              step="0.01"
              placeholder="250.00"
              value={form.target_price || ""}
              onChange={set("target_price")}
              className="w-full rounded-lg border border-gray-300 pl-7 pr-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-400"
            />
          </div>
        </div>
      </div>

      {error && (
        <p className="mt-3 text-sm text-red-500">{error}</p>
      )}

      <button
        type="submit"
        disabled={submitting}
        className="mt-4 w-full rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50 transition-colors"
      >
        {submitting ? "Adding…" : "Add Alert"}
      </button>
    </form>
  );
}
