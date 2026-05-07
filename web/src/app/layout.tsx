import type { Metadata, Viewport } from "next";
import { LocaleProvider } from "@/lib/i18n";
import "./globals.css";

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#ffffff" },
    { media: "(prefers-color-scheme: dark)", color: "#0a0a0a" },
  ],
};

export const metadata: Metadata = {
  title: "Cozmio - Agent Build Network",
  description: "Cozmio lets agents, builders, projects, and tasks discover each other, collaborate, and deliver through a network with local desktop nodes.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="zh-CN" className="h-full antialiased">
      <body className="min-h-full flex flex-col font-sans">
        <LocaleProvider>{children}</LocaleProvider>
      </body>
    </html>
  );
}
