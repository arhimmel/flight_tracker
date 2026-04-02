"use client";

import { useState } from "react";
import toast from "react-hot-toast";

interface Props {
  onSendCode: (email: string) => Promise<void>;
  onLogin: (email: string, otp: string) => Promise<void>;
}

export function LoginForm({ onSendCode, onLogin }: Props) {
  const [step, setStep] = useState<"email" | "otp">("email");
  const [email, setEmail] = useState("");
  const [otp, setOtp] = useState("");
  const [submitting, setSubmitting] = useState(false);

  async function handleEmailSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    try {
      await onSendCode(email);
      setStep("otp");
      toast.success(`Code sent to ${email}`);
    } catch {
      toast.error("Failed to send code. Check your email and try again.");
    } finally {
      setSubmitting(false);
    }
  }

  async function handleOtpSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    try {
      await onLogin(email, otp);
    } catch {
      toast.error("Invalid or expired code.");
      setOtp("");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="mx-auto max-w-sm pt-8">
      <div className="rounded-xl border border-gray-200 bg-white p-8 shadow-sm">
        <h2 className="mb-1 text-lg font-semibold text-gray-800">Sign in</h2>
        <p className="mb-6 text-sm text-gray-500">
          {step === "email"
            ? "Enter your email to receive a one-time login code."
            : `Enter the 6-digit code sent to ${email}.`}
        </p>

        {step === "email" ? (
          <form onSubmit={handleEmailSubmit} className="flex flex-col gap-4">
            <input
              type="email"
              required
              autoFocus
              placeholder="you@example.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="rounded-lg border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-400"
            />
            <button
              type="submit"
              disabled={submitting}
              className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50 transition-colors"
            >
              {submitting ? "Sending…" : "Send code"}
            </button>
          </form>
        ) : (
          <form onSubmit={handleOtpSubmit} className="flex flex-col gap-4">
            <input
              type="text"
              required
              autoFocus
              inputMode="numeric"
              placeholder="000000"
              maxLength={6}
              value={otp}
              onChange={(e) => setOtp(e.target.value.replace(/\D/g, ""))}
              className="rounded-lg border border-gray-300 px-3 py-2 text-center text-2xl tracking-widest focus:outline-none focus:ring-2 focus:ring-blue-400"
            />
            <button
              type="submit"
              disabled={submitting || otp.length !== 6}
              className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50 transition-colors"
            >
              {submitting ? "Verifying…" : "Sign in"}
            </button>
            <button
              type="button"
              onClick={() => { setStep("email"); setOtp(""); }}
              className="text-sm text-gray-500 hover:underline"
            >
              Use a different email
            </button>
          </form>
        )}
      </div>
    </div>
  );
}
