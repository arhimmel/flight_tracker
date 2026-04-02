import { Alert, AuthResponse, CreateAlertPayload } from "./types";

const API_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

const TOKEN_KEY = "flight_tracker_token";

function authHeaders(): HeadersInit {
  const token =
    typeof window !== "undefined" ? localStorage.getItem(TOKEN_KEY) : null;
  return token ? { Authorization: `Bearer ${token}` } : {};
}

// ── Auth ─────────────────────────────────────────────────────────────────────

export async function requestOtp(email: string): Promise<void> {
  const res = await fetch(`${API_URL}/auth/request-otp`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email }),
  });
  if (!res.ok) throw new Error("Failed to send code");
}

export async function verifyOtp(
  email: string,
  otp: string
): Promise<AuthResponse> {
  const res = await fetch(`${API_URL}/auth/verify-otp`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, otp }),
  });
  if (!res.ok) throw new Error("Invalid or expired code");
  return res.json();
}

// ── Alerts ───────────────────────────────────────────────────────────────────

export async function fetchAlerts(): Promise<Alert[]> {
  const res = await fetch(`${API_URL}/alerts`, {
    headers: authHeaders(),
  });
  if (!res.ok) throw new Error("Failed to fetch alerts");
  return res.json();
}

export async function createAlert(payload: CreateAlertPayload): Promise<Alert> {
  const res = await fetch(`${API_URL}/alerts`, {
    method: "POST",
    headers: { "Content-Type": "application/json", ...authHeaders() },
    body: JSON.stringify(payload),
  });
  if (!res.ok) throw new Error("Failed to create alert");
  return res.json();
}

export async function deleteAlert(id: number): Promise<void> {
  const res = await fetch(`${API_URL}/alerts/${id}`, {
    method: "DELETE",
    headers: authHeaders(),
  });
  if (!res.ok) throw new Error("Failed to delete alert");
}

export function getSSEUrl(): string {
  return `${API_URL}/events`;
}
