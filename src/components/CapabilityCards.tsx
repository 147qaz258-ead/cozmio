import { Container } from "@/components/layout/Container";

const PRINCIPLES = [
  {
    eyebrow: "Observed",
    title: "工作痕迹直接进入系统",
    body: "标题停留、封面回看、页面切换、回到创作台，这些都属于工作现场的一部分。Pulseclaw 先接住这些片段，再让后续帮助建立在它们之上。",
    points: ["停留与切换是 raw signal", "窗口与关键帧可回放", "先有证据，再有解释"],
  },
  {
    eyebrow: "Evidence-first",
    title: "证据链站在产品最前面",
    body: "原始记录始终是主层，派生阅读层与候选层保持降权。帮助可以出现，但它必须带着来源一起出现。",
    points: ["raw = source of truth", "derived = candidate only", "任何帮助都能回指证据"],
  },
  {
    eyebrow: "Restrained Help",
    title: "帮助在合适的时机长出来",
    body: "Pulseclaw 让帮助顺着证据自然出现。它不抢在现场之前说话，也不把候选层包装成最后答案。",
    points: ["先保留上下文", "再提出候选帮助", "不假装已理解用户全部目标"],
  },
];

export function CapabilityCards() {
  return (
    <section id="principles" className="pb-8 pt-4 sm:pt-8">
      <Container>
        <div className="grid gap-10 lg:grid-cols-[0.76fr_1.24fr] lg:gap-14">
          <div className="max-w-[30rem]">
            <span className="section-kicker">Design Principles</span>
            <h2 className="mt-6 text-[clamp(2.2rem,4vw,3.35rem)] font-semibold leading-[1.04] text-primary-text">
              让真实上下文，先于提示词进入系统。
            </h2>
            <p className="mt-5 text-[1rem] leading-7 text-secondary-text sm:text-[1.06rem]">
              Pulseclaw 先接住工作现场，再把帮助建立在证据与边界上。
              这套秩序本身，就是产品可信度的一部分。
            </p>
          </div>

          <div className="grid gap-4 lg:grid-cols-3">
            {PRINCIPLES.map((item) => (
              <article
                key={item.title}
                className="surface-panel rounded-[1.75rem] p-5 sm:p-6"
              >
                <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                  {item.eyebrow}
                </div>
                <h3 className="mt-4 text-[1.35rem] font-semibold leading-8 text-primary-text">
                  {item.title}
                </h3>
                <p className="mt-4 text-sm leading-7 text-secondary-text">
                  {item.body}
                </p>

                <div className="mt-6 space-y-2.5">
                  {item.points.map((point) => (
                    <div
                      key={point}
                      className="rounded-2xl border border-black/6 bg-white/76 px-3.5 py-3 text-sm font-medium text-primary-text"
                    >
                      <span className="mr-3 inline-flex h-2.5 w-2.5 rounded-full bg-digital-lavender/80" />
                      {point}
                    </div>
                  ))}
                </div>
              </article>
            ))}
          </div>
        </div>
      </Container>
    </section>
  );
}
