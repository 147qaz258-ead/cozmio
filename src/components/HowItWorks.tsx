import { Container } from "@/components/layout/Container";

const STORY_CHAPTERS = [
  {
    index: "01",
    title: "用户先与世界交互，上下文先开始流动",
    body: "真正的工作上下文往往在语言之前就发生了。阅读、停留、切换、回看，这些信号先出现，提示词却总是更晚。",
    tags: ["dwell", "revisit", "switch"],
  },
  {
    index: "02",
    title: "这段经历被保留成可回放的证据包",
    body: "窗口、关键帧、时间顺序与事件切换被收束成 replayable evidence。这里还没有“理解结论”，只有可信的保留。",
    tags: ["raw", "append-only", "replayable"],
  },
  {
    index: "03",
    title: "候选帮助在证据之后出现",
    body: "只有当证据足以支撑一个上下文包时，AI 才开始给出下一步建议，而且这些建议始终带着来源一起出现。",
    tags: ["candidate", "evidence-backed", "restrained"],
  },
];

export function HowItWorks() {
  return (
    <section id="story" className="py-14 sm:py-18 lg:py-24">
      <Container>
        <div className="surface-panel-strong rounded-[2rem] p-6 sm:p-8 lg:p-10">
          <div className="grid gap-10 lg:grid-cols-[0.8fr_1.2fr] lg:gap-14">
            <div className="max-w-[32rem]">
              <span className="section-kicker">故事结构</span>
              <h2 className="mt-6 text-[clamp(2.2rem,4vw,3.3rem)] font-semibold leading-[1.06] text-primary-text">
                从一个真实片段，长成一条可回放的上下文链。
              </h2>
              <p className="mt-5 text-[1rem] leading-7 text-secondary-text sm:text-[1.06rem]">
                一段阅读、一次切换、一个回看动作，都可以被保留下来，成为后续帮助的前提。
                Pulseclaw 让这个过程具备顺序、边界和回放能力。
              </p>

              <div className="mt-7 rounded-[1.5rem] border border-black/6 bg-white/78 p-5">
                <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                  产品结构
                </div>
                <p className="mt-3 text-sm leading-7 text-primary-text">
                  原始记录、回放能力、候选帮助各有位置。
                  它们一起构成一条完整的产品链路，协同支撑同一个展示系统。
                </p>
              </div>
            </div>

            <div className="space-y-4">
              {STORY_CHAPTERS.map((chapter) => (
                <article
                  key={chapter.index}
                  className="surface-panel rounded-[1.75rem] p-5 sm:p-6"
                >
                  <div className="flex flex-col gap-5 sm:flex-row sm:items-start sm:justify-between">
                    <div className="flex gap-4">
                      <div className="inline-flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl border border-black/6 bg-white/88 text-sm font-semibold text-primary-text shadow-[0_14px_30px_rgba(45,42,38,0.05)]">
                        {chapter.index}
                      </div>
                      <div>
                        <h3 className="text-[1.24rem] font-semibold leading-8 text-primary-text">
                          {chapter.title}
                        </h3>
                        <p className="mt-3 text-sm leading-7 text-secondary-text">
                          {chapter.body}
                        </p>
                      </div>
                    </div>

                    <div className="flex flex-wrap gap-2 sm:max-w-[14rem] sm:justify-end">
                      {chapter.tags.map((tag) => (
                        <span
                          key={tag}
                          className="rounded-full border border-black/6 bg-white/76 px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.14em] text-secondary-text/72"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                  </div>
                </article>
              ))}
            </div>
          </div>
        </div>
      </Container>
    </section>
  );
}
