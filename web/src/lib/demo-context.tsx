"use client";

import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { DEMO_SCRIPT, getNextSignificantStep } from "@/lib/demo-script";

export type DemoMode = "autoplay" | "step-by-step" | "explore-freely";

export interface DemoState {
  mode: DemoMode;
  timelinePosition: number;
  currentStep: number;
  isPlaying: boolean;
  isPaused: boolean;
  totalSteps: number;
  activeEvidenceId: string | null;
  activeNodeId: string | null;
}

export interface DemoContextValue extends DemoState {
  setMode: (mode: DemoMode) => void;
  setTimelinePosition: (position: number) => void;
  setCurrentStep: (step: number) => void;
  play: () => void;
  pause: () => void;
  resume: () => void;
  nextStep: () => void;
  prevStep: () => void;
  reset: () => void;
  setActiveEvidenceId: (id: string | null) => void;
  setActiveNodeId: (id: string | null) => void;
}

const DEMO_DURATION = 50;

function getStepIndexAtTime(time: number) {
  return DEMO_SCRIPT.reduce((activeIndex, step, index) => {
    if (step.timestamp <= time) {
      return index;
    }
    return activeIndex;
  }, 0);
}

function buildStatePatchFromTime(position: number) {
  const clamped = Math.max(0, Math.min(DEMO_DURATION, position));
  const stepIndex = getStepIndexAtTime(clamped);
  const step = DEMO_SCRIPT[stepIndex] ?? DEMO_SCRIPT[0];

  return {
    timelinePosition: clamped,
    currentStep: stepIndex,
    activeEvidenceId: step?.evidenceId ?? null,
    activeNodeId: step?.action?.type === "highlight_node" ? step.action.target : null,
  };
}

const DEFAULT_STATE: DemoState = {
  mode: "autoplay",
  timelinePosition: 0,
  currentStep: 0,
  isPlaying: true,
  isPaused: false,
  totalSteps: DEMO_SCRIPT.length,
  activeEvidenceId: DEMO_SCRIPT[0]?.evidenceId ?? null,
  activeNodeId:
    DEMO_SCRIPT[0]?.action?.type === "highlight_node" ? DEMO_SCRIPT[0].action.target : null,
};

const DemoContext = createContext<DemoContextValue | undefined>(undefined);

export function DemoProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<DemoState>(DEFAULT_STATE);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const pauseTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const setMode = useCallback((mode: DemoMode) => {
    setState((prev) => ({
      ...prev,
      mode,
      isPlaying: mode === "autoplay",
      isPaused: false,
    }));
  }, []);

  const setTimelinePosition = useCallback((position: number) => {
    setState((prev) => ({
      ...prev,
      ...buildStatePatchFromTime(position),
    }));
  }, []);

  const setCurrentStep = useCallback((step: number) => {
    const clampedIndex = Math.max(0, Math.min(DEMO_SCRIPT.length - 1, step));
    const targetStep = DEMO_SCRIPT[clampedIndex];
    setState((prev) => ({
      ...prev,
      ...buildStatePatchFromTime(targetStep.timestamp),
    }));
  }, []);

  const play = useCallback(() => {
    setState((prev) => ({
      ...prev,
      mode: "autoplay",
      isPlaying: true,
      isPaused: false,
    }));
  }, []);

  const pause = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isPaused: true,
    }));
  }, []);

  const resume = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isPlaying: true,
      isPaused: false,
    }));
  }, []);

  const nextStep = useCallback(() => {
    setState((prev) => {
      const nextStepIndex = Math.min(prev.currentStep + 1, DEMO_SCRIPT.length - 1);
      return {
        ...prev,
        ...buildStatePatchFromTime(DEMO_SCRIPT[nextStepIndex].timestamp),
      };
    });
  }, []);

  const prevStep = useCallback(() => {
    setState((prev) => {
      const prevStepIndex = Math.max(prev.currentStep - 1, 0);
      return {
        ...prev,
        ...buildStatePatchFromTime(DEMO_SCRIPT[prevStepIndex].timestamp),
      };
    });
  }, []);

  const reset = useCallback(() => {
    setState(DEFAULT_STATE);
  }, []);

  const setActiveEvidenceId = useCallback((id: string | null) => {
    setState((prev) => ({
      ...prev,
      activeEvidenceId: id,
    }));
  }, []);

  const setActiveNodeId = useCallback((id: string | null) => {
    setState((prev) => ({
      ...prev,
      activeNodeId: id,
    }));
  }, []);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.code === "Space" && state.mode === "autoplay") {
        event.preventDefault();
        if (state.isPlaying && !state.isPaused) {
          pause();
          return;
        }
        resume();
      }

      if (state.mode === "step-by-step") {
        if (event.code === "ArrowRight") {
          event.preventDefault();
          nextStep();
        }
        if (event.code === "ArrowLeft") {
          event.preventDefault();
          prevStep();
        }
      }

      if (state.mode === "explore-freely") {
        if (event.code === "ArrowRight") {
          event.preventDefault();
          setTimelinePosition(state.timelinePosition + 5);
        }
        if (event.code === "ArrowLeft") {
          event.preventDefault();
          setTimelinePosition(state.timelinePosition - 5);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [nextStep, pause, prevStep, resume, setTimelinePosition, state]);

  useEffect(() => {
    if (state.mode !== "autoplay" || !state.isPlaying || state.isPaused) {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
      return;
    }

    const tickInterval = 90;
    const timeIncrement = 0.38;

    timerRef.current = setInterval(() => {
      setState((prev) => {
        const newTime = prev.timelinePosition + timeIncrement;
        const nextSignificant = getNextSignificantStep(prev.timelinePosition);

        if (
          nextSignificant &&
          newTime >= nextSignificant.timestamp - 0.25 &&
          newTime <= nextSignificant.timestamp + 0.2
        ) {
          return {
            ...prev,
            ...buildStatePatchFromTime(nextSignificant.timestamp),
            isPaused: true,
          };
        }

        if (newTime >= DEMO_DURATION) {
          return {
            ...prev,
            ...buildStatePatchFromTime(DEMO_DURATION),
            isPlaying: false,
            isPaused: false,
          };
        }

        return {
          ...prev,
          ...buildStatePatchFromTime(newTime),
        };
      });
    }, tickInterval);

    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [state.isPaused, state.isPlaying, state.mode]);

  useEffect(() => {
    if (state.mode !== "autoplay" || !state.isPlaying || !state.isPaused) {
      if (pauseTimeoutRef.current) {
        clearTimeout(pauseTimeoutRef.current);
        pauseTimeoutRef.current = null;
      }
      return;
    }

    pauseTimeoutRef.current = setTimeout(() => {
      resume();
    }, 900);

    return () => {
      if (pauseTimeoutRef.current) {
        clearTimeout(pauseTimeoutRef.current);
        pauseTimeoutRef.current = null;
      }
    };
  }, [resume, state.isPaused, state.isPlaying, state.mode]);

  const value = useMemo<DemoContextValue>(
    () => ({
      ...state,
      setMode,
      setTimelinePosition,
      setCurrentStep,
      play,
      pause,
      resume,
      nextStep,
      prevStep,
      reset,
      setActiveEvidenceId,
      setActiveNodeId,
    }),
    [
      nextStep,
      pause,
      play,
      prevStep,
      reset,
      resume,
      setActiveEvidenceId,
      setActiveNodeId,
      setCurrentStep,
      setMode,
      setTimelinePosition,
      state,
    ]
  );

  return <DemoContext.Provider value={value}>{children}</DemoContext.Provider>;
}

export function useDemo() {
  const context = useContext(DemoContext);
  if (!context) {
    throw new Error("useDemo must be used within a DemoProvider");
  }
  return context;
}