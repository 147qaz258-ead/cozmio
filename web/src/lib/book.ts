import fs from "fs";
import path from "path";
import matter from "gray-matter";
import { remark } from "remark";
import html from "remark-html";
import gfm from "remark-gfm";

const contentDir = path.join(process.cwd(), "content/book");

export interface Chapter {
  slug: string;
  volume: string;
  volumeIndex: number;
  chapterIndex: number;
  title: string;
  status: string;
  words: string;
  excerpt: string;
  contentHtml: string;
  toc: TocItem[];
}

export interface Volume {
  id: string;
  index: number;
  title: string;
  chapters: ChapterMeta[];
}

export interface ChapterMeta {
  slug: string;
  title: string;
  chapterIndex: number;
  status: string;
}

export interface TocItem {
  id: string;
  text: string;
  level: number;
}

export interface BookStats {
  totalChapters: number;
  volumes: number;
  totalWords: string;
}

const volumeTitles: Record<string, string> = {
  vol1: "第一卷：入门",
  vol2: "第二卷：通道与连接",
  vol3: "第三卷：Agent 深度实践",
  vol4: "第四卷：高级配置",
  vol5: "第五卷：附录",
};

/**
 * 获取所有卷的结构
 */
export function getAllVolumes(): Volume[] {
  const volumes: Volume[] = [];
  const volumeDirs = fs.readdirSync(contentDir).sort();

  volumeDirs.forEach((volId, idx) => {
    const volPath = path.join(contentDir, volId);
    if (!fs.statSync(volPath).isDirectory()) return;

    const chapters = getVolumeChapters(volId);
    volumes.push({
      id: volId,
      index: idx + 1,
      title: volumeTitles[volId] || volId,
      chapters,
    });
  });

  return volumes;
}

/**
 * 获取某卷的所有章节元信息
 */
function getVolumeChapters(volumeId: string): ChapterMeta[] {
  const volPath = path.join(contentDir, volumeId);
  const files = fs.readdirSync(volPath).filter((f) => f.endsWith(".md")).sort();

  return files.map((file) => {
    const filePath = path.join(volPath, file);
    const fileContent = fs.readFileSync(filePath, "utf-8");
    const { data, content } = matter(fileContent);

    const slug = file.replace(".md", "");

    // 如果没有 frontmatter，从第一行提取标题
    let title = data.title;
    if (!title) {
      const firstLine = content.split("\n")[0];
      const titleMatch = firstLine.match(/^#\s+(.+)/);
      if (titleMatch) {
        title = titleMatch[1].replace(/\s*·\s*/g, " - ").trim();
      } else {
        title = slug;
      }
    }

    // 从标题提取章节编号
    let chapterIndex = data.chapter || 0;
    if (chapterIndex === 0 && title) {
      const chapterMatch = title.match(/第\s*(\d+)\s*章/);
      if (chapterMatch) {
        chapterIndex = parseInt(chapterMatch[1], 10);
      }
    }

    return {
      slug,
      title,
      chapterIndex,
      status: data.status || "draft",
    };
  });
}

/**
 * 获取所有章节（用于首页展示）
 */
export function getAllChapters(): ChapterMeta[] {
  const volumes = getAllVolumes();
  return volumes.flatMap((vol) => vol.chapters);
}

/**
 * 根据卷和 slug 获取章节内容
 */
export async function getChapterBySlug(volumeId: string, slug: string): Promise<Chapter | null> {
  const filePath = path.join(contentDir, volumeId, `${slug}.md`);

  if (!fs.existsSync(filePath)) {
    return null;
  }

  const fileContent = fs.readFileSync(filePath, "utf-8");
  const { data, content } = matter(fileContent);

  // 渲染 Markdown
  const processedContent = await remark()
    .use(gfm)
    .use(html, { sanitize: false })
    .process(content);

  const contentHtml = processedContent.toString();

  // 提取 excerpt（前 200 字）
  const excerpt = content.slice(0, 200).replace(/\n/g, " ").trim();

  // 提取 TOC（从 Markdown 中提取标题）
  const toc = extractToc(content);

  // 计算卷索引
  const volumeDirs = fs.readdirSync(contentDir).sort();
  const volumeIndex = volumeDirs.indexOf(volumeId) + 1;

  return {
    slug,
    volume: volumeId,
    volumeIndex,
    chapterIndex: data.chapter || 0,
    title: data.title || slug,
    status: data.status || "draft",
    words: data.words || "",
    excerpt,
    contentHtml,
    toc,
  };
}

/**
 * 从 Markdown 内容提取目录
 */
function extractToc(content: string): TocItem[] {
  const toc: TocItem[] = [];
  const lines = content.split("\n");

  lines.forEach((line) => {
    const match = line.match(/^(#{1,3})\s+(.+)$/);
    if (match) {
      const level = match[1].length;
      const text = match[2].trim();
      const id = text
        .toLowerCase()
        .replace(/[^\w\u4e00-\u9fa5]+/g, "-")
        .replace(/^-|-$/g, "");

      toc.push({ id, text, level });
    }
  });

  return toc;
}

/**
 * 获取相邻章节（上一篇/下一篇）
 */
export async function getAdjacentChapters(
  volumeId: string,
  currentSlug: string
): Promise<{ prev: ChapterMeta | null; next: ChapterMeta | null }> {
  const volumes = getAllVolumes();
  const allChapters = volumes.flatMap((vol) =>
    vol.chapters.map((ch) => ({ ...ch, volumeId: vol.id }))
  );

  const currentIndex = allChapters.findIndex(
    (ch) => ch.volumeId === volumeId && ch.slug === currentSlug
  );

  if (currentIndex === -1) {
    return { prev: null, next: null };
  }

  const prev = currentIndex > 0 ? allChapters[currentIndex - 1] : null;
  const next = currentIndex < allChapters.length - 1 ? allChapters[currentIndex + 1] : null;

  return {
    prev: prev ? { slug: prev.slug, title: prev.title, chapterIndex: prev.chapterIndex, status: prev.status } : null,
    next: next ? { slug: next.slug, title: next.title, chapterIndex: next.chapterIndex, status: next.status } : null,
  };
}

/**
 * 获取书籍统计信息
 */
export function getBookStats(): BookStats {
  const volumes = getAllVolumes();
  const totalChapters = volumes.reduce((sum, vol) => sum + vol.chapters.length, 0);

  return {
    totalChapters,
    volumes: volumes.length,
    totalWords: "约 40 万字",
  };
}
