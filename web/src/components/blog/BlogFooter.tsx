import Link from "next/link";

/**
 * 博客专用 Footer
 * 简洁设计，专注于书籍信息
 */
export function BlogFooter() {
  return (
    <footer className="border-t border-black/6 bg-white/40">
      <div className="mx-auto max-w-6xl px-6 py-12">
        <div className="grid gap-8 md:grid-cols-3">
          {/* Brand */}
          <div>
            <div className="flex items-center gap-2 mb-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-gradient-to-br from-mist-blue to-sage-green text-white font-bold text-sm">
                O
              </div>
              <span className="font-semibold text-primary-text">OpenClaw Book</span>
            </div>
            <p className="text-sm text-secondary-text leading-relaxed">
              从零开始掌握 AI Agent 框架。<br />
              开源、自托管、面向实践者。
            </p>
          </div>

          {/* Links */}
          <div>
            <h4 className="text-xs font-semibold uppercase tracking-wider text-secondary-text/66 mb-4">
              快速导航
            </h4>
            <ul className="space-y-2">
              <li>
                <Link href="/book/vol1/ch-01" className="text-sm text-secondary-text hover:text-mist-blue transition-colors">
                  开始阅读 →
                </Link>
              </li>
              <li>
                <Link href="https://github.com/147qaz258-ead/Pulseclaw" className="text-sm text-secondary-text hover:text-mist-blue transition-colors">
                  GitHub 仓库
                </Link>
              </li>
              <li>
                <Link href="https://openclaw.ai" className="text-sm text-secondary-text hover:text-mist-blue transition-colors">
                  OpenClaw 官网
                </Link>
              </li>
            </ul>
          </div>

          {/* Author */}
          <div>
            <h4 className="text-xs font-semibold uppercase tracking-wider text-secondary-text/66 mb-4">
              作者
            </h4>
            <p className="text-sm text-secondary-text">
              jinhongw840@gmail.com
            </p>
            <p className="text-xs text-secondary-text/60 mt-2">
              © 2026 OpenClaw Book. All rights reserved.
            </p>
          </div>
        </div>
      </div>
    </footer>
  );
}