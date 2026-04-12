"use client";

import Link from "next/link";
import { useState, useEffect } from "react";

interface TocItem {
  id: string;
  text: string;
  level: number;
}

interface TocSidebarProps {
  items: TocItem[];
  activeId?: string;
}

/**
 * 目录侧边栏组件
 */
export function TocSidebar({ items, activeId }: TocSidebarProps) {
  const [currentActiveId, setCurrentActiveId] = useState(activeId);

  // 监听滚动，更新当前活跃标题
  useEffect(() => {
    const handleScroll = () => {
      const headings = items.map((item) => document.getElementById(item.id));
      const scrollPosition = window.scrollY + 100;

      for (let i = headings.length - 1; i >= 0; i--) {
        const heading = headings[i];
        if (heading && heading.offsetTop <= scrollPosition) {
          setCurrentActiveId(items[i].id);
          break;
        }
      }
    };

    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, [items]);

  return (
    <nav className="toc-sidebar">
      <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66 mb-4">
        目录
      </div>
      <ul className="space-y-2">
        {items.map((item) => (
          <li
            key={item.id}
            className={`${item.level === 1 ? "" : item.level === 2 ? "ml-2" : "ml-4"}`}
          >
            <Link
              href={`#${item.id}`}
              className={`block py-1.5 text-sm transition-colors ${
                currentActiveId === item.id
                  ? "text-primary-text font-medium"
                  : "text-secondary-text/74 hover:text-primary-text"
              }`}
            >
              {item.text}
            </Link>
          </li>
        ))}
      </ul>
    </nav>
  );
}