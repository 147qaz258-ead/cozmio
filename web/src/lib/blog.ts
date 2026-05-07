export interface BlogSection {
  heading: string;
  paragraphs: string[];
}

export interface BlogPost {
  slug: string;
  title: string;
  summary: string;
  publishedAt: string;
  readingTime: string;
  category: string;
  kicker: string;
  intro: string;
  tags: string[];
  sections: BlogSection[];
}

export const BLOG_POSTS: BlogPost[] = [
  {
    slug: "context-before-prompt",
    title: "上下文应该先于提示词进入系统",
    summary:
      "如果智能体总要等一句 prompt 才开始理解工作，它就永远落后于真实现场。Pulseclaw 的起点是把刚发生过的上下文先接进来。",
    publishedAt: "2026-04-04",
    readingTime: "6 min",
    category: "Product Thinking",
    kicker: "Writing 01",
    intro:
      "很多人以为智能体用不好，是因为模型还不够强。但真正更常见的问题是，用户已经在工作里表达了很多东西，系统却什么都没接住。",
    tags: ["context", "prompting", "product"],
    sections: [
      {
        heading: "语言总是晚于现场",
        paragraphs: [
          "当一个人反复看一篇文章的标题、停留在封面、切回自己的素材库、又回到编辑器，这些动作本身已经在表达偏好、目标和下一步方向。",
          "传统 AI 交互却要求用户再把这一切压缩成一句 prompt。问题不只是麻烦，而是用户在翻译的过程中会损失大量上下文颗粒度。",
        ],
      },
      {
        heading: "系统先接住，再谈帮助",
        paragraphs: [
          "Pulseclaw 的思路不是更早抢答，而是更早接住工作现场。先把窗口切换、停留、关键帧、终端文本这些原始片段保留下来，再让后续帮助建立在这些证据之上。",
          "这样一来，帮助不再像凭空出现的猜测，而是从一段已经存在的经历里自然长出来。",
        ],
      },
      {
        heading: "这也是产品边界",
        paragraphs: [
          "上下文先于提示词，不等于系统可以越级替用户下结论。它接住的是刚发生过的轨迹，不是神秘的读心能力。",
          "真正可信的地方，在于用户始终可以回到证据链本身，重新阅读、重新组织、重新判断。",
        ],
      },
    ],
  },
  {
    slug: "raw-evidence-before-summaries",
    title: "为什么原始证据应该站在最前面",
    summary:
      "总结很方便，但总结天然会压缩细节。Pulseclaw 把原始记录放在最前面，是为了让系统始终保有可验证性。",
    publishedAt: "2026-04-04",
    readingTime: "5 min",
    category: "Product Boundary",
    kicker: "Writing 02",
    intro:
      "很多产品会把“智能整理”当成价值中心，但一旦整理结果覆盖了原始现场，可信度就开始下降。尤其是和 AI 结合之后，这个问题会被放大。",
    tags: ["evidence", "replay", "trust"],
    sections: [
      {
        heading: "总结很强，也很危险",
        paragraphs: [
          "总结可以把长链路压缩成更短的信息单元，这在效率上非常有吸引力。但只要总结被误读成真相，它就开始带来风险。",
          "对于调试、研究、写作这类高上下文密度的任务，用户真正需要的不是一段看起来正确的说法，而是一条随时可以回去核对的轨迹。",
        ],
      },
      {
        heading: "Replay 比结论更重要",
        paragraphs: [
          "Pulseclaw 把 replay 放在结构中央，是因为回放能力决定了帮助是否还能被追问。你可以回到某个时间点、某个窗口、某条错误文本，再看后来出现的候选解释是否站得住。",
          "这让系统不是只会给答案，而是保留了重新阅读问题的能力。",
        ],
      },
      {
        heading: "候选说明始终从属",
        paragraphs: [
          "派生层和候选层当然有价值。它们负责组织、提示、加速理解，但它们必须带着边界出现。",
          "原始记录是真相层，候选解释是工作层。把两者分开，系统才不会在“看起来聪明”的时候失去可信度。",
        ],
      },
    ],
  },
  {
    slug: "why-pulseclaw-starts-on-desktop",
    title: "为什么 Pulseclaw 先从桌面端开始",
    summary:
      "Pulseclaw 的起点不是浏览器里的表单，而是工作真正发生的地方。桌面端更接近真实上下文，也更适合本地优先的产品形态。",
    publishedAt: "2026-04-04",
    readingTime: "6 min",
    category: "Platform",
    kicker: "Writing 03",
    intro:
      "如果产品的目标是接住真实工作上下文，那它最开始就不应该只站在一个网页输入框前面。很多关键片段根本不在那里发生。",
    tags: ["desktop", "local-first", "product"],

    sections: [
      {
        heading: "工作不是在单一网页里完成的",
        paragraphs: [
          "写作要看文章、素材库和编辑器，调试要看代码、终端和文档，研究要在多个窗口之间不断切换。真正的工作上下文天然就是跨界面的。",
          "如果系统只能看到网页里的一个输入框，它看到的永远只是已经被压缩过的结果。",
        ],
      },
      {
        heading: "桌面端让本地优先成立",
        paragraphs: [
          "Pulseclaw 强调本地优先，不是出于口号，而是因为上下文捕获这件事本来就更适合站在设备侧完成。越靠近现场，越能保留顺序、边界和细节。",
          "当产品在桌面端运行时，网站就不需要假装自己是产品本体。网站应该是正式官网、公开 demo 入口和思考窗口。",
        ],
      },
      {
        heading: "网站的职责是公开展示，而不是伪装成 app",
        paragraphs: [
          "这也是为什么 Pulseclaw 当前官网不强行摆一个假的登录入口。现在更重要的是把产品表达、演示体系、系统边界和设计语言收成一个完整的公开站点。",
          "当真正的桌面入口准备好时，官网再自然接上下载、候补或正式产品入口就可以了。",
        ],
      },
    ],
  },
  {
    slug: "i-built-a-desktop-ai-that-watches-my-screen",
    title: "我用 AI 造了个桌面助手，它能感知我在做什么",
    summary:
      "上周三下午 3 点，我正在赶 PPT，老板 4 点要。我忘了上周会议说的那个关键数字。找了 20 分钟找不到。然后 Pulseclaw 弹出来了。3 分钟搞定。",
    publishedAt: "2026-04-11",
    readingTime: "5 min",
    category: "User Story",
    kicker: "Writing 04",
    intro:
      "上周三下午 3 点，我差点把键盘砸了。我正在赶一个 PPT 的最后几页——老板 4 点要。我刚把数据图表做好，正准备写结论，突然......我忘了上周会议上老板说的那个关键数字。",
    tags: ["story", "user-story", "context-awareness"],
    sections: [
      {
        heading: "3 分钟搞定",
        paragraphs: [
          "我翻了 20 分钟邮件、微信、笔记 app、飞书文档......找不到。",
          "然后 Pulseclaw 弹出来了。它显示：'你在找关于 Q3 毛利率的数据？上周三会议纪要里有。' 我点击，直接跳到那一行。",
        ],
      },
      {
        heading: "不是又一个聊天机器人",
        paragraphs: [
          "我知道你在想什么——'这不就是 AI 助手吗？Copilot、ChatGPT、Claude 都能做这事。' 不，不一样。最大的区别是：我没有问它。它自己感知到了。",
        ],
      },
      {
        heading: "一个从不打扰你的超级助理",
        paragraphs: [
          "Pulseclaw 是一个桌面 AI 助手，但它和你想的不一样。它不是另一个需要你不断提示的聊天机器人。它在后台安静地观察你的屏幕活动，理解你当前的工作状态，只有在信号足够强时才弹出帮你。就像一个从不打扰你的超级助理，只在你真正需要的时候出现。",
          "就像一个从不打扰你的超级助理，只在你真正需要的时候出现。",
        ],
      },
      {
        heading: "47 个标签页的笑话",
        paragraphs: [
          "有一次，Pulseclaw 弹出来想帮我'打开记事本'。我当时已经开了 47 个标签页，同时跑着 VS Code、Slack、4 个飞书文档、3 个 Excel......它居然以为我需要记事本。",
          "我笑了。但后来我想明白了——这不是 AI 傻，这是它在尝试理解我的工作流。它在等一个足够确定的机会。相比一个总在你不需要的时候叨叨的 AI，这个至少知道什么时候该闭嘴。",
        ],
      },
      {
        heading: "隐私这件事，我想清楚了",
        paragraphs: [
          "我知道你在想什么——'这不就是监控我的电脑吗？' Pulseclaw 的所有数据都在本地处理。你的桌面活动不会上传到任何云端。没有服务器，没有 API 调用记录。它就安静地在你电脑上运行，像一个真正值得信任的本地工具。",
        ],
      },
      {
        heading: "为什么我做了这个",
        paragraphs: [
          "因为我受够了 AI 助手。不是 AI 不够好——是它们总是在错的时间出现。你正在赶进度，它弹出说'需要帮助吗？' 你终于进入心流状态了，它提示你'你已经工作 2 小时了，要不要休息一下？'",
          "这些 AI 像是那种特别热情但不懂察言观色的实习生——有用，但总是在最糟糕的时候凑过来。我想要的是：一个真正懂得什么时候该出现的 AI。所以我造了 Pulseclaw。",
        ],
      },
    ],
  },
];

export function getAllPosts() {
  return BLOG_POSTS.toSorted((a, b) => (a.publishedAt < b.publishedAt ? 1 : -1));
}

export function getPostBySlug(slug: string) {
  return BLOG_POSTS.find((post) => post.slug === slug);
}
