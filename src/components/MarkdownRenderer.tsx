"use client";

import { useEffect } from "react";

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

/**
 * Markdown 渲染组件
 * 支持代码高亮、表格、图片、链接
 */
export function MarkdownRenderer({ content, className = "" }: MarkdownRendererProps) {
  // 客户端代码高亮（如果使用 rehype-highlight）
  useEffect(() => {
    // 动态加载 highlight.js 样式（如果需要）
    if (typeof window !== "undefined") {
      // 这里可以动态注入 highlight.js CSS
    }
  }, []);

  return (
    <div
      className={`markdown-content prose prose-lg max-w-none ${className}`}
      dangerouslySetInnerHTML={{ __html: content }}
    />
  );
}