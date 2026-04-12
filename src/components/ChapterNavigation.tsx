import Link from "next/link";

interface ChapterMeta {
  slug: string;
  title: string;
  chapterIndex: number;
  status: string;
}

interface ChapterNavigationProps {
  volumeId: string;
  prev: ChapterMeta | null;
  next: ChapterMeta | null;
}

/**
 * 章节导航组件（上一篇/下一篇）
 */
export function ChapterNavigation({ volumeId, prev, next }: ChapterNavigationProps) {
  return (
    <div className="flex items-center justify-between gap-4 py-8 border-t border-black/6">
      {prev ? (
        <Link
          href={`/book/${volumeId}/${prev.slug}`}
          className="flex-1 surface-panel rounded-[1.4rem] p-4 transition-transform hover:-translate-y-0.5"
        >
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
            上一篇
          </div>
          <div className="mt-2 text-sm font-medium text-primary-text">{prev.title}</div>
        </Link>
      ) : (
        <div className="flex-1" />
      )}

      {next ? (
        <Link
          href={`/book/${volumeId}/${next.slug}`}
          className="flex-1 surface-panel rounded-[1.4rem] p-4 transition-transform hover:-translate-y-0.5 text-right"
        >
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
           下一篇
          </div>
          <div className="mt-2 text-sm font-medium text-primary-text">{next.title}</div>
        </Link>
      ) : (
        <div className="flex-1" />
      )}
    </div>
  );
}