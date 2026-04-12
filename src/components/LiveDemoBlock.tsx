"use client";

import { useState, useEffect, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Container } from "@/components/layout/Container";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

// Demo phases
type DemoPhase = "observing" | "recording" | "replaying";

// Simulated window activities - 用户在 VS Code 编写代码
const windowActivities = [
  { title: "VS Code - debugger.ts", icon: "code", timestamp: "14:32:01" },
  { title: "Chrome - React Docs", icon: "browser", timestamp: "14:31:45" },
  { title: "Terminal - npm run dev", icon: "terminal", timestamp: "14:30:22" },
  { title: "Console - Error Output", icon: "chat", timestamp: "14:29:58" },
];

// Simulated context inference - 记录用户活动轨迹
const contextInference = [
  { label: "当前活动", value: "调试代码中的错误" },
  { label: "工作状态", value: "遇到问题，正在排查" },
  { label: "相关上下文", value: "控制台有错误日志" },
  { label: "时间", value: "调试中 · 15 分钟" },
];

// Simulated suggestion - 提议查看相关日志
const suggestion = {
  title: "发现相关日志",
  content: "检测到控制台有与当前调试相关的错误信息。是否需要查看日志详情？",
  actions: ["查看日志", "稍后", "忽略"],
};

const phaseConfig = {
  observing: {
    label: "观察中",
    color: "mist-blue",
    duration: 3000,
  },
  recording: {
    label: "记录中",
    color: "digital-lavender",
    duration: 2000,
  },
  replaying: {
    label: "回放中",
    color: "sage-green",
    duration: 4000,
  },
};

export function LiveDemoBlock() {
  const [phase, setPhase] = useState<DemoPhase>("observing");
  const [isPlaying, setIsPlaying] = useState(true);
  const [activeIndex, setActiveIndex] = useState(0);

  // Auto-cycle through phases
  useEffect(() => {
    if (!isPlaying) return;

    const phases: DemoPhase[] = ["observing", "recording", "replaying"];
    const durations = [3000, 2000, 4000];

    let currentIndex = phases.indexOf(phase);
    const timeout = setTimeout(() => {
      currentIndex = (currentIndex + 1) % phases.length;
      setPhase(phases[currentIndex]);
      if (currentIndex === 0) {
        setActiveIndex((prev) => (prev + 1) % windowActivities.length);
      }
    }, durations[currentIndex]);

    return () => clearTimeout(timeout);
  }, [phase, isPlaying]);

  const handleManualTrigger = useCallback(() => {
    setIsPlaying(false);
    setPhase("observing");
    setTimeout(() => {
      setPhase("recording");
      setTimeout(() => {
        setPhase("replaying");
        setTimeout(() => {
          setIsPlaying(true);
        }, 4000);
      }, 2000);
    }, 3000);
  }, []);

  return (
    <section className="py-16 md:py-24 bg-gradient-to-b from-warm-white to-warm-card">
      <Container>
        <div className="text-center mb-12">
          <h2 className="text-2xl md:text-3xl font-bold text-primary-text mb-4">
            看见它如何工作
          </h2>
          <p className="text-secondary-text max-w-xl mx-auto">
            观察整个流程：从感知环境到记录轨迹，再到回放查看
          </p>
        </div>

        {/* Demo visualization */}
        <div className="max-w-4xl mx-auto">
          {/* Phase indicator */}
          <div className="flex justify-center gap-4 mb-8">
            {(["observing", "recording", "replaying"] as DemoPhase[]).map((p) => (
              <div
                key={p}
                className={`flex items-center gap-2 px-4 py-2 rounded-full transition-all ${
                  phase === p
                    ? `bg-${phaseConfig[p].color}/20 border border-${phaseConfig[p].color}`
                    : "bg-warm-card/50"
                }`}
              >
                <div
                  className={`w-3 h-3 rounded-full ${
                    phase === p ? `bg-${phaseConfig[p].color}` : "bg-secondary-text/30"
                  }`}
                />
                <span
                  className={`text-sm ${
                    phase === p ? "text-primary-text font-medium" : "text-secondary-text"
                  }`}
                >
                  {phaseConfig[p].label}
                </span>
              </div>
            ))}
          </div>

          {/* Main demo card */}
          <Card className="bg-white/90 border-border/40 shadow-xl overflow-hidden">
            <CardContent className="p-6 md:p-8">
              {/* Phase content */}
              <div className="space-y-4">
                <AnimatePresence mode="wait">
                  {phase === "observing" && (
                    <motion.div
                      key="observing"
                      initial={{ opacity: 0, x: -20 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: 20 }}
                      transition={{ duration: 0.3 }}
                    >
                    <div className="flex items-center gap-2 mb-4">
                      <div className="w-4 h-4 rounded-full bg-mist-blue animate-pulse" />
                      <span className="text-sm text-mist-blue font-medium">实时窗口活动</span>
                    </div>
                    <div className="grid gap-2">
                      {windowActivities.map((activity, idx) => (
                        <div
                          key={idx}
                          className={`flex items-center gap-3 p-3 rounded-lg transition-all ${
                            idx === activeIndex
                              ? "bg-mist-blue/10 border border-mist-blue/30"
                              : "bg-warm-card/30"
                          }`}
                        >
                          <div className={`w-8 h-8 rounded bg-${activity.icon === "code" ? "mist-blue" : activity.icon === "browser" ? "digital-lavender" : activity.icon === "terminal" ? "sage-green" : "warm-card"}/30 flex items-center justify-center`}>
                            {activity.icon === "code" && (
                              <svg className="w-4 h-4 text-mist-blue" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                              </svg>
                            )}
                            {activity.icon === "browser" && (
                              <svg className="w-4 h-4 text-digital-lavender" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                              </svg>
                            )}
                            {activity.icon === "terminal" && (
                              <svg className="w-4 h-4 text-sage-green" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                              </svg>
                            )}
                            {activity.icon === "chat" && (
                              <svg className="w-4 h-4 text-secondary-text" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                              </svg>
                            )}
                          </div>
                          <div className="flex-1">
                            <div className="text-sm text-primary-text">{activity.title}</div>
                            <div className="text-xs text-secondary-text">{activity.timestamp}</div>
                          </div>
                          {idx === activeIndex && (
                            <div className="w-2 h-2 rounded-full bg-mist-blue animate-pulse" />
                          )}
                        </div>
                      ))}
                    </div>
                    </motion.div>
                  )}

                {phase === "recording" && (
                  <motion.div
                    key="recording"
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: 20 }}
                    transition={{ duration: 0.3 }}
                  >
                    <div className="flex items-center gap-2 mb-4">
                      <div className="w-4 h-4 rounded-full bg-digital-lavender animate-pulse" />
                      <span className="text-sm text-digital-lavender font-medium">证据采集</span>
                    </div>
                    <div className="grid gap-3">
                      {contextInference.map((item, idx) => (
                        <motion.div
                          key={idx}
                          initial={{ opacity: 0, y: 10 }}
                          animate={{ opacity: 1, y: 0 }}
                          transition={{ delay: idx * 0.1 }}
                          className="flex items-center gap-4 p-3 rounded-lg bg-digital-lavender/10"
                        >
                          <div className="text-xs text-secondary-text w-24">{item.label}</div>
                          <div className="flex-1 text-sm text-primary-text font-medium">{item.value}</div>
                        </motion.div>
                      ))}
                    </div>
                    <div className="mt-4 p-3 rounded-lg bg-digital-lavender/20 border border-digital-lavender/30">
                      <div className="text-xs text-digital-lavender mb-1">采集记录</div>
                      <div className="text-sm text-primary-text">
                        窗口切换序列已记录，时间戳已保存，活动轨迹完整
                      </div>
                    </div>
                  </motion.div>
                )}

                {phase === "replaying" && (
                  <motion.div
                    key="replaying"
                    initial={{ opacity: 0, x: -20 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: 20 }}
                    transition={{ duration: 0.3 }}
                  >
                    <div className="flex items-center gap-2 mb-4">
                      <div className="w-4 h-4 rounded-full bg-sage-green" />
                      <span className="text-sm text-sage-green font-medium">轨迹回放</span>
                    </div>
                    <div className="p-4 rounded-lg bg-sage-green/10 border border-sage-green/30">
                      <div className="flex items-start gap-3">
                        <div className="w-10 h-10 rounded-full bg-sage-green/20 flex items-center justify-center">
                          <svg className="w-5 h-5 text-sage-green" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.314-5.686l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.954-.383-1.869-1.053-2.548l-.547-.547z" />
                          </svg>
                        </div>
                        <div className="flex-1">
                          <div className="text-sm font-medium text-primary-text mb-2">
                            {suggestion.title}
                          </div>
                          <div className="text-sm text-secondary-text mb-4">
                            {suggestion.content}
                          </div>
                          <div className="flex gap-2">
                            {suggestion.actions.map((action, idx) => (
                              <Button
                                key={idx}
                                size="sm"
                                variant={idx === 0 ? "default" : "outline"}
                                className={idx === 0 ? "bg-sage-green hover:bg-sage-green/90 text-white" : "border-sage-green/30 text-sage-green"}
                              >
                                {action}
                              </Button>
                            ))}
                          </div>
                        </div>
                      </div>
                    </div>
                    <div className="mt-4 text-center text-xs text-secondary-text">
                      点击任意时间点可跳转回放，查看当时发生的活动
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
              </div>
            </CardContent>
          </Card>

          {/* Manual trigger button */}
          <div className="mt-6 text-center">
            <Button
              variant="outline"
              onClick={handleManualTrigger}
              className="border-mist-blue text-mist-blue hover:bg-mist-blue/10"
            >
              {isPlaying ? "暂停自动播放" : "手动触发演示"}
            </Button>
          </div>
        </div>
      </Container>
    </section>
  );
}