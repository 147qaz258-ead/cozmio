import { ReactNode } from "react";

export default function UseLayout({ children }: { children: ReactNode }) {
  return <div className="min-h-screen bg-warm-50">{children}</div>;
}