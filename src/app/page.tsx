import type { Metadata } from "next";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { HeroSection } from "@/components/HeroSection";
import { CapabilityCards } from "@/components/CapabilityCards";
import { HowItWorks } from "@/components/HowItWorks";
import { CTASection } from "@/components/sections/CTASection";

export const metadata: Metadata = {
  description:
    "Pulseclaw 桌面端工具：自动捕获工作上下文，在证据支撑下提供 AI 帮助。接住上下文，让 AI 在有边界的前提下开口。",
};

export default function Home() {
  return (
    <div className="flex flex-col min-h-screen bg-warm-white">
      <Header />
      <main className="flex-1">
        {/* Hero is the immediate first-screen focus */}
        <HeroSection />

        {/* Supporting content below - secondary sections */}
        <div className="bg-gradient-to-b from-warm-white to-warm-card/20">
          <CapabilityCards />
          <HowItWorks />
          <CTASection />
        </div>
      </main>
      <Footer />
    </div>
  );
}