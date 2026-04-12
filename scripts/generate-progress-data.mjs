#!/usr/bin/env node
/**
 * generate-progress-data.mjs
 * Reads git log from current repo, outputs progress data to public/data/progress.json
 *
 * Usage: node scripts/generate-progress-data.mjs
 */

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const OUT_DIR = path.join(ROOT, "public", "data");
const OUT_FILE = path.join(OUT_DIR, "progress.json");

const DAYS_TO_SHOW = 21;
const COMMITS_PER_DAY = 4;
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
  return ROOT;
}

function shouldIncludeFile(filePath) {
  return !IGNORE_PREFIXES.some((prefix) => filePath.startsWith(prefix));
}

function parseStat(value) {
  if (value === "-" || value.trim() === "") return 0;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) ? parsed : 0;
}

function collapseArea(filePath) {
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

function readCommitRecords() {
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

    const commits = [];
    let current = null;

    for (const line of output.split(/\r?\n/)) {
      if (!line.trim()) continue;

      if (line.startsWith("__COMMIT__|")) {
        if (current) {
          current.filesChanged = current.files.length;
          commits.push(current);
        }

        const parts = line.split("|");
        const sha = parts[1];
        const date = parts[2];
        const subject = parts.slice(3).join("|").trim();
        current = {
          sha,
          date,
          subject,
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
  } catch (err) {
    console.error("Failed to read git log:", err.message);
    return [];
  }
}

function summarizeDailyProgress(commits) {
  const grouped = new Map();

  for (const commit of commits) {
    const list = grouped.get(commit.date) ?? [];
    list.push(commit);
    grouped.set(commit.date, list);
  }

  return Array.from(grouped.entries())
    .sort((a, b) => b[0].localeCompare(a[0]))
    .slice(0, DAYS_TO_SHOW)
    .map(([date, dayCommits]) => {
      const additions = dayCommits.reduce((sum, c) => sum + c.additions, 0);
      const deletions = dayCommits.reduce((sum, c) => sum + c.deletions, 0);
      const filesChanged = dayCommits.reduce((sum, c) => sum + c.filesChanged, 0);

      const areaScores = new Map();
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
        highlights: dayCommits.slice(0, COMMITS_PER_DAY).map((c) => c.subject),
        areas,
        commits: dayCommits.slice(0, COMMITS_PER_DAY),
      };
    });
}

function buildRecentStats(dailyProgress) {
  const recentWindow = dailyProgress.slice(0, 7);
  const recentAdditions = recentWindow.reduce((sum, d) => sum + d.additions, 0);
  const recentDeletions = recentWindow.reduce((sum, d) => sum + d.deletions, 0);
  const recentCommitCount = recentWindow.reduce((sum, d) => sum + d.commitCount, 0);
  return { recentAdditions, recentDeletions, recentCommitCount };
}

function main() {
  const commits = readCommitRecords();
  const dailyProgress = summarizeDailyProgress(commits);
  const recentStats = buildRecentStats(dailyProgress);

  const data = {
    generatedAt: new Date().toISOString(),
    dailyProgress,
    recentStats,
  };

  if (!existsSync(OUT_DIR)) {
    mkdirSync(OUT_DIR, { recursive: true });
  }

  writeFileSync(OUT_FILE, JSON.stringify(data, null, 2), "utf8");
  console.log(`Wrote ${dailyProgress.length} days of data to ${OUT_FILE}`);
  console.log(
    `Recent stats: +${recentStats.recentAdditions} / -${recentStats.recentDeletions} / ${recentStats.recentCommitCount} commits`,
  );
}

main();
