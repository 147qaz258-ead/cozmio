"use client";

export function PrivacyNote() {
  return (
    <section className="py-12 px-4 md:px-8 bg-digital-lavender/10">
      <div className="max-w-2xl mx-auto text-center">
        <h3 className="text-lg font-medium text-primary-text mb-4">我们的理念</h3>
        <div className="text-sm text-secondary-text space-y-2">
          <p>
            Pulseclaw 主要在你的本地运行。你的代码、你的上下文、你的决策都留在你身边。
          </p>
          <p>
            我们在采集证据的地方使用技术，但不假装它是魔法。它是一个工具，在你了解它的工作方式时效果最好。
          </p>
          <p>
            无遥测，无隐藏数据收集。我们相信工具应该服务用户，而不是监控他们。
          </p>
        </div>
      </div>
    </section>
  );
}