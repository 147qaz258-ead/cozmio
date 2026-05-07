import Link from "next/link";

interface BlogHeaderProps {
  currentPath?: string;
}

/**
 * 博客专用 Header
 * 独立于 Pulseclaw 落地页，有自己的品牌风格
 */
export function BlogHeader({ currentPath }: BlogHeaderProps) {
  return (
    <header className="sticky top-0 z-50 border-b border-black/6 bg-warm-white/80 backdrop-blur-xl">
      <div className="mx-auto max-w-6xl px-6 py-4">
        <div className="flex items-center justify-between">
          {/* Logo & Brand */}
          <Link href="/blog" className="group flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-gradient-to-br from-mist-blue to-sage-green text-white font-bold text-lg shadow-md">
              O
            </div>
            <div>
              <div className="text-lg font-semibold text-primary-text">OpenClaw Book</div>
              <div className="text-xs text-secondary-text">AI Agent 框架实践指南</div>
            </div>
          </Link>

          {/* Navigation */}
          <nav className="hidden md:flex items-center gap-1">
            <NavLink href="/blog" active={currentPath === "/blog"}>
              全部章节
            </NavLink>
            <NavLink href="#about" active={currentPath === "/about"}>
              关于本书
            </NavLink>
            <NavLink href="https://github.com/147qaz258-ead/Pulseclaw" external>
              GitHub
            </NavLink>
          </nav>

          {/* Actions */}
          <div className="flex items-center gap-3">
            <Link
              href="/"
              className="rounded-full border border-black/8 bg-white/60 px-4 py-2 text-sm font-medium text-secondary-text transition-all hover:bg-white hover:border-black/12 hover:text-primary-text"
            >
              ← 返回主页
            </Link>
          </div>
        </div>
      </div>
    </header>
  );
}

function NavLink({ 
  href, 
  children, 
  active, 
  external 
}: { 
  href: string; 
  children: React.ReactNode; 
  active?: boolean;
  external?: boolean;
}) {
  const className = `rounded-full px-4 py-2 text-sm font-medium transition-colors ${
    active 
      ? "bg-mist-blue/10 text-mist-blue" 
      : "text-secondary-text hover:bg-black/4 hover:text-primary-text"
  }`;

  if (external) {
    return (
      <a href={href} target="_blank" rel="noopener noreferrer" className={className}>
        {children}
      </a>
    );
  }

  return (
    <Link href={href} className={className}>
      {children}
    </Link>
  );
}