import { Alert, CreateAlertPayload } from "./types";

const API_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

export async function fetchAlerts(): Promise<Alert[]> {
  const res = await fetch(`${API_URL}/alerts`);
  if (!res.ok) throw new Error("Failed to fetch alerts");
  return res.json();
}

export async function createAlert(payload: CreateAlertPayload): Promise<Alert> {
  const res = await fetch(`${API_URL}/alerts`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!res.ok) throw new Error("Failed to create alert");
  return res.json();
}

export async function deleteAlert(id: number): Promise<void> {
  const res = await fetch(`${API_URL}/alerts/${id}`, { method: "DELETE" });
  if (!res.ok) throw new Error("Failed to delete alert");
}

export function getSSEUrl(): string {
  return `${API_URL}/events`;
}
