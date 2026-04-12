import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DATA_FILE = path.join(__dirname, "..", "..", "public", "data", "progress.json");

function formatNum(value: number) {
  return new Intl.NumberFormat("zh-CN").format(value);
}

function formatDate(dateStr: string) {
  const d = new Date(`${dateStr}T00:00:00`);
  return new Intl.DateTimeFormat("zh-CN", { month: "short", day: "numeric" }).format(d);
}

function barHeight(net: number, maxNet: number) {
  if (maxNet === 0) return 4;
  return Math.max(4, Math.round((Math.abs(net) / maxNet) * 120));
}

interface CommitRecord {
  sha: string;
  date: string;
  subject: string;
  additions: number;
  deletions: number;
  filesChanged: number;
  files: { path: string; additions: number; deletions: number }[];
}

interface DailyProgressDay {
  date: string;
  commitCount: number;
  additions: number;
  deletions: number;
  filesChanged: number;
  netLines: number;
  highlights: string[];
  areas: string[];
  commits: CommitRecord[];
}

interface RecentStats {
  recentAdditions: number;
  recentDeletions: number;
  recentCommitCount: number;
}

interface ProgressJson {
  generatedAt: string;
  dailyProgress: DailyProgressDay[];
  recentStats: RecentStats;
}

export interface ProgressPageData {
  projectName: string;
  codename: string;
  generatedAt: string;
  tasklistVersion: string;
  totalTasks: number;
  passedTasks: number;
  activeTasks: number;
  blockedTasks: number;
  completionRate: number;
  recentAdditions: number;
  recentDeletions: number;
  recentCommitCount: number;
  dailyProgress: DailyProgressDay[];
  laneSnapshots: never[];
  currentFrontier: never[];
}

export function getProgressPageData(): ProgressPageData {
  let raw: ProgressJson | null = null;

  try {
    if (fs.existsSync(DATA_FILE)) {
      const content = fs.readFileSync(DATA_FILE, "utf8");
      raw = JSON.parse(content) as ProgressJson;
    }
  } catch {
    // fall through to empty state
  }

  if (!raw) {
    return {
      projectName: "Pulseclaw",
      codename: "Pulseclaw",
      generatedAt: new Date().toISOString(),
      tasklistVersion: "unavailable",
      totalTasks: 0,
      passedTasks: 0,
      activeTasks: 0,
      blockedTasks: 0,
      completionRate: 0,
      recentAdditions: 0,
      recentDeletions: 0,
      recentCommitCount: 0,
      dailyProgress: [],
      laneSnapshots: [],
      currentFrontier: [],
    };
  }

  const maxNet = Math.max(
    ...raw.dailyProgress.map((d) => Math.abs(d.netLines)),
    1,
  );

  const totalAdditions = raw.dailyProgress.reduce((s, d) => s + d.additions, 0);
  const totalDeletions = raw.dailyProgress.reduce((s, d) => s + d.deletions, 0);

  return {
    projectName: "Pulseclaw",
    codename: "Pulseclaw",
    generatedAt: raw.generatedAt,
    tasklistVersion: "git-driven",
    totalTasks: 0,
    passedTasks: 0,
    activeTasks: 0,
    blockedTasks: 0,
    completionRate: 0,
    recentAdditions: raw.recentStats.recentAdditions,
    recentDeletions: raw.recentStats.recentDeletions,
    recentCommitCount: raw.recentStats.recentCommitCount,
    dailyProgress: raw.dailyProgress,
    laneSnapshots: [],
    currentFrontier: [],
  };
}

export { formatNum, formatDate, barHeight };
