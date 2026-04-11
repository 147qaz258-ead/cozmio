import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

type TaskStatus =
  | "todo"
  | "in_progress"
  | "implemented_unverified"
  | "passed"
  | "failed"
  | "blocked";

type VerificationStatus = "not_run" | "passed" | "failed" | "blocked";

interface TaskRecord {
  id: string;
  lane: string;
  title: string;
  status: TaskStatus;
  verification_status?: VerificationStatus;
  depends_on?: string[];
  notes?: string;
}

interface LaneRecord {
  id: string;
  name: string;
  description: string;
}

interface TasklistFile {
  version: string;
  lanes: LaneRecord[];
  tasks: TaskRecord[];
}

interface CommitFileStat {
  path: string;
  additions: number;
  deletions: number;
}

interface CommitRecord {
  sha: string;
  date: string;
  subject: string;
  additions: number;
  deletions: number;
  filesChanged: number;
  files: CommitFileStat[];
}

export interface DailyProgressDay {
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

export interface LaneSnapshot {
  id: string;
  name: string;
  description: string;
  total: number;
  passed: number;
  active: number;
  blocked: number;
  tasks: TaskRecord[];
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
  laneSnapshots: LaneSnapshot[];
  currentFrontier: TaskRecord[];
}

const TASKLIST_RELATIVE_PATH = [".omc", "pulseclaw_master_tasklist_v4.json"];
const DAYS_TO_SHOW = 21;
const COMMITS_PER_DAY = 4;
const RECENT_WINDOW = 7;
const GIT_MAX_BUFFER = 24 * 1024 * 1024;
const IGNORE_PREFIXES = [
  "target/",
  "web/node_modules/",
  "web/.next/",
  "web/out/",
  ".tmp-verify/",
  ".verification/",
  "verification/",
];

function getRepoRoot() {
  return path.resolve(process.cwd(), "..");
}

function tasklistPath() {
  return path.join(getRepoRoot(), ...TASKLIST_RELATIVE_PATH);
}

function safeReadTasklist(): TasklistFile | null {
  try {
    if (!fs.existsSync(tasklistPath())) {
      return null;
    }
    const raw = fs.readFileSync(tasklistPath(), "utf8");
    return JSON.parse(raw) as TasklistFile;
  } catch {
    return null;
  }
}

function shouldIncludeFile(filePath: string) {
  return !IGNORE_PREFIXES.some((prefix) => filePath.startsWith(prefix));
}

function parseStat(value: string) {
  if (value === "-" || value.trim() === "") return 0;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) ? parsed : 0;
}

function collapseArea(filePath: string) {
  const segments = filePath.split("/");
  if (segments[0] === "web") {
    if (segments[1] === "src" && segments[2]) return `web/src/${segments[2]}`;
    if (segments[1]) return `web/${segments[1]}`;
  }

  if (segments[0] === "src" && segments[1]) return `src/${segments[1]}`;
  if (segments[0] === "tests" && segments[1]) return `tests/${segments[1]}`;
  if (segments[0]?.startsWith(".")) {
    if (segments[1]) return `${segments[0]}/${segments[1]}`;
    return segments[0];
  }

  if (segments[1]) return `${segments[0]}/${segments[1]}`;
  return segments[0] ?? filePath;
}

function readCommitRecords(): CommitRecord[] {
  try {
    const output = execFileSync(
      "git",
      [
        "log",
        `--since=${DAYS_TO_SHOW + 14} days ago`,
        "--date=short",
        "--pretty=format:__COMMIT__|%H|%ad|%s",
        "--numstat",
      ],
      {
        cwd: getRepoRoot(),
        encoding: "utf8",
        maxBuffer: GIT_MAX_BUFFER,
      },
    );

    const commits: CommitRecord[] = [];
    let current: CommitRecord | null = null;

    for (const line of output.split(/\r?\n/)) {
      if (!line.trim()) continue;

      if (line.startsWith("__COMMIT__|")) {
        if (current) {
          current.filesChanged = current.files.length;
          commits.push(current);
        }

        const [, sha, date, ...subjectParts] = line.split("|");
        current = {
          sha,
          date,
          subject: subjectParts.join("|").trim(),
          additions: 0,
          deletions: 0,
          filesChanged: 0,
          files: [],
        };
        continue;
      }

      if (!current) continue;

      const parts = line.split("\t");
      if (parts.length < 3) continue;

      const filePath = parts.slice(2).join("\t").trim();
      if (!shouldIncludeFile(filePath)) continue;

      const additions = parseStat(parts[0]);
      const deletions = parseStat(parts[1]);

      current.files.push({ path: filePath, additions, deletions });
      current.additions += additions;
      current.deletions += deletions;
    }

    if (current) {
      current.filesChanged = current.files.length;
      commits.push(current);
    }

    return commits;
  } catch {
    return [];
  }
}

function summarizeDailyProgress(commits: CommitRecord[]): DailyProgressDay[] {
  const grouped = new Map<string, CommitRecord[]>();

  for (const commit of commits) {
    const list = grouped.get(commit.date) ?? [];
    list.push(commit);
    grouped.set(commit.date, list);
  }

  return Array.from(grouped.entries())
    .sort((a, b) => b[0].localeCompare(a[0]))
    .slice(0, DAYS_TO_SHOW)
    .map(([date, dayCommits]) => {
      const additions = dayCommits.reduce((sum, commit) => sum + commit.additions, 0);
      const deletions = dayCommits.reduce((sum, commit) => sum + commit.deletions, 0);
      const filesChanged = dayCommits.reduce((sum, commit) => sum + commit.filesChanged, 0);

      const areaScores = new Map<string, number>();
      for (const commit of dayCommits) {
        for (const file of commit.files) {
          const area = collapseArea(file.path);
          const score = file.additions + file.deletions || 1;
          areaScores.set(area, (areaScores.get(area) ?? 0) + score);
        }
      }

      const areas = Array.from(areaScores.entries())
        .sort((a, b) => b[1] - a[1])
        .slice(0, 4)
        .map(([area]) => area);

      return {
        date,
        commitCount: dayCommits.length,
        additions,
        deletions,
        filesChanged,
        netLines: additions - deletions,
        highlights: dayCommits.slice(0, COMMITS_PER_DAY).map((commit) => commit.subject),
        areas,
        commits: dayCommits.slice(0, COMMITS_PER_DAY),
      };
    });
}

function statusWeight(task: TaskRecord) {
  const order: Record<TaskStatus, number> = {
    in_progress: 0,
    blocked: 1,
    failed: 2,
    implemented_unverified: 3,
    todo: 4,
    passed: 5,
  };

  return order[task.status] ?? 99;
}

export function getProgressPageData(): ProgressPageData {
  const tasklist = safeReadTasklist();
  const commits = readCommitRecords();
  const dailyProgress = summarizeDailyProgress(commits);

  if (!tasklist) {
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

  const laneSnapshots = tasklist.lanes.map((lane) => {
    const tasks = tasklist.tasks
      .filter((task) => task.lane === lane.id)
      .sort((a, b) => a.id.localeCompare(b.id, undefined, { numeric: true }));

    return {
      id: lane.id,
      name: lane.name,
      description: lane.description,
      total: tasks.length,
      passed: tasks.filter((task) => task.status === "passed").length,
      active: tasks.filter((task) => task.status === "in_progress").length,
      blocked: tasks.filter(
        (task) =>
          task.status === "blocked" || task.verification_status === "blocked",
      ).length,
      tasks,
    };
  });

  const totalTasks = tasklist.tasks.length;
  const passedTasks = tasklist.tasks.filter((task) => task.status === "passed").length;
  const activeTasks = tasklist.tasks.filter((task) =>
    ["in_progress", "implemented_unverified"].includes(task.status),
  ).length;
  const blockedTasks = tasklist.tasks.filter(
    (task) =>
      task.status === "blocked" || task.verification_status === "blocked",
  ).length;

  const currentFrontier = tasklist.tasks
    .filter((task) => task.status !== "passed")
    .sort((a, b) => {
      if (a.lane !== b.lane) return a.lane.localeCompare(b.lane);
      const weightDelta = statusWeight(a) - statusWeight(b);
      if (weightDelta !== 0) return weightDelta;
      return a.id.localeCompare(b.id, undefined, { numeric: true });
    })
    .slice(0, 6);

  const recentWindow = dailyProgress.slice(0, RECENT_WINDOW);
  const recentAdditions = recentWindow.reduce((sum, day) => sum + day.additions, 0);
  const recentDeletions = recentWindow.reduce((sum, day) => sum + day.deletions, 0);
  const recentCommitCount = recentWindow.reduce((sum, day) => sum + day.commitCount, 0);

  return {
    projectName: "Pulseclaw",
    codename: "Pulseclaw",
    generatedAt: new Date().toISOString(),
    tasklistVersion: tasklist.version,
    totalTasks,
    passedTasks,
    activeTasks,
    blockedTasks,
    completionRate: totalTasks > 0 ? passedTasks / totalTasks : 0,
    recentAdditions,
    recentDeletions,
    recentCommitCount,
    dailyProgress,
    laneSnapshots,
    currentFrontier,
  };
}
