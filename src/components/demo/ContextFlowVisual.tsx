"use client";

import { motion } from "framer-motion";
import { cn } from "@/lib/utils";

interface ContextFlowVisualProps {
  stepIndex: number;
  sceneTitle: string;
  sceneDescription: string;
  sceneSignals: string[];
  capsuleTitle: string;
  capsuleDescription: string;
  capsuleItems: string[];
  outputTitle: string;
  outputDescription: string;
  outputItems: string[];
  variant?: "compact" | "feature";
}

export function ContextFlowVisual({
  stepIndex,
  sceneTitle,
  sceneDescription,
  sceneSignals,
  capsuleTitle,
  capsuleDescription,
  capsuleItems,
  outputTitle,
  outputDescription,
  outputItems,
  variant = "feature",
}: ContextFlowVisualProps) {
  const capsuleVisible = stepIndex >= 1;
  const outputVisible = stepIndex >= 2;
  const outputActive = stepIndex >= 3;
  const isCompact = variant === "compact";

  return (
    <div
      className={cn(
        "relative overflow-hidden rounded-[1.6rem] border border-black/6 bg-white/82 p-4 shadow-[0_18px_44px_rgba(45,42,38,0.06)]",
        isCompact ? "lg:p-4" : "lg:p-5"
      )}
    >
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute left-[10%] top-[14%] rounded-full border border-mist-blue/12 bg-white/72 px-3 py-1.5 text-[10px] font-semibold uppercase tracking-[0.18em] text-mist-blue shadow-[0_10px_20px_rgba(45,42,38,0.05)]">
          seed
        </div>
        <motion.div
          className="absolute left-[16%] top-[50%] h-5 w-5 -translate-y-1/2 rounded-full bg-[radial-gradient(circle,rgba(123,158,172,0.98),rgba(184,169,201,0.72),rgba(156,175,136,0.22))] shadow-[0_0_28px_rgba(123,158,172,0.45)]"
          animate={
            outputVisible
              ? {
                  x: [0, 210, 470],
                  y: [0, -14, 4],
                  opacity: [0, 1, 1, 0],
                  scale: [0.72, 1.18, 1, 0.7],
                }
              : capsuleVisible
                ? {
                    x: [0, 210],
                    opacity: [0, 1, 0],
                    scale: [0.72, 1.18, 0.8],
                  }
                : {
                    opacity: [0.2, 0.85, 0.2],
                    scale: [0.88, 1.04, 0.88],
                  }
          }
          transition={{
            duration: outputVisible ? 2.8 : 2.1,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        />
        <motion.div
          className="absolute left-[18%] top-[57%] h-3.5 w-3.5 -translate-y-1/2 rounded-full bg-[radial-gradient(circle,rgba(156,175,136,0.96),rgba(123,158,172,0.55),rgba(255,255,255,0.1))] shadow-[0_0_22px_rgba(156,175,136,0.34)]"
          animate={
            outputVisible
              ? {
                  x: [0, 180, 440],
                  y: [0, 16, -6],
                  opacity: [0, 0.9, 0.9, 0],
                  scale: [0.7, 1, 0.92, 0.7],
                }
              : capsuleVisible
                ? {
                    x: [0, 180],
                    y: [0, 10],
                    opacity: [0, 0.8, 0],
                  }
                : { opacity: 0 }
          }
          transition={{
            duration: 2.35,
            repeat: Infinity,
            ease: "easeInOut",
            delay: 0.25,
          }}
        />
      </div>

      <div className="pointer-events-none absolute inset-x-8 top-[50%] hidden -translate-y-1/2 lg:block">
        <div className="relative h-px">
          <motion.div
            className="signal-beam absolute left-[22%] h-px origin-left rounded-full"
            style={{ width: "18%" }}
            animate={{ opacity: capsuleVisible ? 1 : 0.2, scaleX: capsuleVisible ? 1 : 0.18 }}
            transition={{ duration: 0.45, ease: "easeOut" }}
          />
          <motion.div
            className="signal-beam absolute left-[58%] h-px origin-left rounded-full"
            style={{ width: "16%" }}
            animate={{ opacity: outputVisible ? 1 : 0.16, scaleX: outputVisible ? 1 : 0.18 }}
            transition={{ duration: 0.45, ease: "easeOut" }}
          />

          <motion.span
            className="absolute left-[22%] top-1/2 inline-flex h-3 w-3 -translate-y-1/2 rounded-full bg-mist-blue shadow-[0_0_16px_rgba(123,158,172,0.46)]"
            animate={
              capsuleVisible
                ? {
                    x: ["0%", "520%"],
                    opacity: [0, 1, 0],
                    scale: [0.8, 1, 0.85],
                  }
                : { opacity: 0 }
            }
            transition={{
              duration: 1.35,
              repeat: Infinity,
              ease: "easeInOut",
            }}
          />
          <motion.span
            className="absolute left-[58%] top-1/2 inline-flex h-3 w-3 -translate-y-1/2 rounded-full bg-sage-green shadow-[0_0_16px_rgba(156,175,136,0.42)]"
            animate={
              outputVisible
                ? {
                    x: ["0%", "420%"],
                    opacity: [0, 1, 0],
                    scale: [0.8, 1, 0.85],
                  }
                : { opacity: 0 }
            }
            transition={{
              duration: 1.15,
              repeat: Infinity,
              ease: "easeInOut",
              delay: 0.2,
            }}
          />
        </div>
      </div>

      <div
        className={cn(
          "grid gap-4 lg:grid-cols-[1.02fr_0.72fr_0.92fr] lg:items-center",
          isCompact && "lg:grid-cols-[1fr_0.68fr_0.92fr]"
        )}
      >
        <div className="relative rounded-[1.3rem] border border-black/6 bg-[linear-gradient(135deg,rgba(123,158,172,0.14),rgba(255,255,255,0.92),rgba(184,169,201,0.1))] p-4">
          <div className="pointer-events-none absolute inset-4 overflow-hidden rounded-[1.1rem]">
            <motion.div
              className="scan-shimmer absolute inset-y-0 -left-1/3 w-1/2"
              animate={{ x: stepIndex >= 0 ? ["0%", "260%"] : "0%" }}
              transition={{ duration: 2.6, repeat: Infinity, ease: "linear" }}
            />
          </div>
          <div className="relative rounded-[1.1rem] border border-white/70 bg-white/88 p-4 shadow-[0_12px_30px_rgba(45,42,38,0.05)]">
            <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/66">
              Scene
            </div>
            <div className={cn("mt-3 font-semibold text-primary-text", isCompact ? "text-[1.15rem] leading-7" : "text-[1.35rem] leading-8")}>
              {sceneTitle}
            </div>
            <p className="mt-3 text-sm leading-7 text-secondary-text">{sceneDescription}</p>

            <div className="mt-4 grid gap-2.5">
              {sceneSignals.map((signal, index) => (
                <motion.div
                  key={signal}
                  className={cn(
                    "rounded-2xl border px-3 py-2.5 text-sm",
                    index <= stepIndex
                      ? "border-black/6 bg-white text-primary-text shadow-[0_12px_24px_rgba(45,42,38,0.04)]"
                      : "border-transparent bg-warm-card/62 text-secondary-text/72"
                  )}
                  animate={index <= stepIndex ? { y: [0, -2, 0] } : { y: 0 }}
                  transition={{ duration: 1.8, repeat: Infinity, ease: "easeInOut", delay: index * 0.12 }}
                >
                  <span
                    className={cn(
                      "mr-3 inline-flex h-2.5 w-2.5 rounded-full",
                      index <= stepIndex ? "bg-mist-blue ambient-pulse" : "bg-secondary-text/24"
                    )}
                  />
                  {signal}
                </motion.div>
              ))}
            </div>
          </div>
        </div>

        <div className="relative flex min-h-[17rem] items-center justify-center rounded-[1.4rem] border border-black/6 bg-[linear-gradient(180deg,rgba(255,255,255,0.95),rgba(246,243,237,0.92))] px-4 py-5">
          <motion.div
            className="absolute inset-0 rounded-[1.4rem]"
            animate={{
              opacity: capsuleVisible ? [0.18, 0.35, 0.18] : 0.1,
            }}
            transition={{ duration: 2.4, repeat: Infinity, ease: "easeInOut" }}
            style={{
              background:
                "radial-gradient(circle at center, rgba(123,158,172,0.16), transparent 52%), radial-gradient(circle at center, rgba(184,169,201,0.12), transparent 72%)",
            }}
          />

          <motion.div
            className="relative flex h-[11.5rem] w-[11.5rem] items-center justify-center rounded-full border border-mist-blue/18 bg-white/86 shadow-[0_20px_44px_rgba(45,42,38,0.08)]"
            initial={{ scale: 0.86, opacity: 0.22 }}
            animate={{
              scale: capsuleVisible ? [1, 1.06, 1] : [0.92, 0.96, 0.92],
              opacity: capsuleVisible ? 1 : 0.68,
              boxShadow: capsuleVisible
                ? [
                    "0 12px 28px rgba(45,42,38,0.06)",
                    "0 24px 50px rgba(123,158,172,0.14)",
                    "0 12px 28px rgba(45,42,38,0.06)",
                  ]
                : "0 12px 28px rgba(45,42,38,0.06)",
            }}
            transition={{ duration: 2.6, repeat: Infinity, ease: "easeInOut" }}
          >
            <motion.div
              className="absolute inset-3 rounded-full border border-dashed border-digital-lavender/26"
              animate={{ rotate: capsuleVisible ? 360 : 0 }}
              transition={{ duration: 8, repeat: Infinity, ease: "linear" }}
            />
            <motion.div
              className="absolute inset-6 rounded-full border border-sage-green/20"
              animate={{ rotate: capsuleVisible ? -360 : 0 }}
              transition={{ duration: 10, repeat: Infinity, ease: "linear" }}
            />

            <div className="relative z-10 text-center">
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/66">
                当前工作片段
              </div>
              <div className="mt-2 text-lg font-semibold text-primary-text">{capsuleTitle}</div>
              <div className="mt-2 text-xs leading-6 text-secondary-text">{capsuleDescription}</div>
            </div>
          </motion.div>

          <div className="pointer-events-none absolute inset-0">
            {capsuleItems.slice(0, 4).map((item, index) => {
              const positions = [
                "left-[10%] top-[16%]",
                "right-[8%] top-[22%]",
                "left-[8%] bottom-[22%]",
                "right-[10%] bottom-[16%]",
              ];
              return (
                <motion.div
                  key={item}
                  className={cn(
                    "absolute rounded-full border border-black/6 bg-white/82 px-3 py-1.5 text-[11px] font-medium text-primary-text shadow-[0_10px_20px_rgba(45,42,38,0.04)]",
                    positions[index]
                  )}
                  animate={
                    capsuleVisible
                      ? {
                          opacity: [0.5, 1, 0.5],
                          y: [0, index % 2 === 0 ? -5 : 5, 0],
                        }
                      : { opacity: 0.18, y: 0 }
                  }
                  transition={{ duration: 2.2, repeat: Infinity, ease: "easeInOut", delay: index * 0.18 }}
                >
                  {item}
                </motion.div>
              );
            })}
          </div>
        </div>

        <div className="relative rounded-[1.3rem] border border-black/6 bg-[linear-gradient(180deg,rgba(255,255,255,0.92),rgba(245,242,236,0.92))] p-4">
          <div className="pointer-events-none absolute inset-4 overflow-hidden rounded-[1.1rem]">
            <motion.div
              className="scan-shimmer absolute inset-y-0 -left-1/3 w-1/2"
              animate={outputVisible ? { x: ["0%", "250%"] } : { x: "-10%" }}
              transition={{ duration: 2.15, repeat: Infinity, ease: "linear", delay: 0.15 }}
            />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/66">
                {outputTitle}
              </div>
              <div className="mt-2 text-sm leading-7 text-secondary-text">{outputDescription}</div>
            </div>
            <span className="rounded-full border border-digital-lavender/18 bg-digital-lavender/10 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-digital-lavender">
              candidate
            </span>
          </div>

          <div className="mt-4 space-y-2.5">
            {outputItems.map((item, index) => (
              <motion.div
                key={item}
                className={cn(
                  "relative overflow-hidden rounded-2xl px-3 py-3 text-sm",
                  outputVisible && index <= stepIndex
                    ? "bg-sage-green/10 text-primary-text"
                    : "bg-warm-card/65 text-secondary-text/74"
                )}
                initial={false}
                animate={
                  outputVisible && index <= stepIndex
                    ? {
                        opacity: 1,
                        x: 0,
                        scale: outputActive ? [1, 1.01, 1] : 1,
                      }
                    : { opacity: 0.55, x: 8, scale: 1 }
                }
                transition={{
                  duration: 0.35,
                  ease: "easeOut",
                  scale: { duration: 1.8, repeat: Infinity, ease: "easeInOut", delay: index * 0.14 },
                }}
              >
                {outputVisible && index <= stepIndex && (
                  <motion.span
                    className="absolute inset-y-0 left-0 w-1 rounded-full bg-[linear-gradient(180deg,rgba(123,158,172,0.9),rgba(156,175,136,0.9))]"
                    animate={{ opacity: [0.65, 1, 0.65] }}
                    transition={{ duration: 1.5, repeat: Infinity, ease: "easeInOut", delay: index * 0.12 }}
                  />
                )}
                {item}
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
