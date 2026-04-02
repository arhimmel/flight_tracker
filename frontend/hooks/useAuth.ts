"use client";

import { useCallback, useEffect, useState } from "react";
import { verifyOtp, requestOtp } from "@/lib/api";

const TOKEN_KEY = "flight_tracker_token";
const EMAIL_KEY = "flight_tracker_email";

export function useAuth() {
  const [token, setToken] = useState<string | null>(null);
  const [email, setEmail] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const storedToken = localStorage.getItem(TOKEN_KEY);
    const storedEmail = localStorage.getItem(EMAIL_KEY);
    if (storedToken && storedEmail) {
      setToken(storedToken);
      setEmail(storedEmail);
    }
    setLoading(false);
  }, []);

  const sendCode = useCallback(async (emailAddr: string) => {
    await requestOtp(emailAddr);
  }, []);

  const login = useCallback(async (emailAddr: string, otp: string) => {
    const result = await verifyOtp(emailAddr, otp);
    localStorage.setItem(TOKEN_KEY, result.token);
    localStorage.setItem(EMAIL_KEY, result.email);
    setToken(result.token);
    setEmail(result.email);
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(EMAIL_KEY);
    setToken(null);
    setEmail(null);
  }, []);

  return {
    token,
    email,
    isAuthenticated: !!token,
    loading,
    sendCode,
    login,
    logout,
  };
}
