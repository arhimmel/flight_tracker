export type AlertStatus = "active" | "triggered" | "expired";

export interface Alert {
  id: number;
  flight_number: string;
  flight_date: string;
  origin: string;
  destination: string;
  target_price: number;
  current_price: number | null;
  status: AlertStatus;
  created_at: string;
  last_checked: string | null;
  notified_at: string | null;
}

export interface CreateAlertPayload {
  flight_number: string;
  flight_date: string;
  origin: string;
  destination: string;
  target_price: number;
}

export interface AlertEvent {
  alert_id: number;
  flight_number: string;
  flight_date: string;
  origin: string;
  destination: string;
  target_price: number;
  current_price: number;
}
