import type { Metadata } from "next";
import { Toaster } from "react-hot-toast";
import "./globals.css";

export const metadata: Metadata = {
  title: "Flight Tracker",
  description: "Get notified when flight prices drop",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-gray-50 text-gray-900 antialiased">
        <header className="border-b border-gray-200 bg-white px-6 py-4">
          <h1 className="text-xl font-bold tracking-tight">✈ Flight Tracker</h1>
        </header>
        <main className="mx-auto max-w-4xl px-4 py-8">{children}</main>
        <Toaster position="top-right" />
      </body>
    </html>
  );
}
