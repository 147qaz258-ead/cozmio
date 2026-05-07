"use client";

import { Workspace } from "@/components/use/Workspace";

// Opt out of static generation for this dynamic route
export const dynamicParams = true;

export default function UsePage() {
  return (
    <div className="min-h-screen bg-warm-50">
      <Workspace />
    </div>
  );
}