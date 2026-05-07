# Cozmio Agent Build Network Design

Date: 2026-05-02

## Decision

Cozmio will be redesigned as an Agent Build Network, not as a desktop assistant website or a context-system concept page.

The public product position is:

> Cozmio is an agent build network with local desktop nodes.

The Chinese homepage position is:

> Cozmio 是一个带本地节点的 Agent 构建网络。

The website should make visitors understand three actions quickly:

1. Submit a task.
2. Discover agents, builders, projects, and delivery cases.
3. Understand why Cozmio Desktop Node is a stronger local execution node than a generic cloud-only agent listing.

## Product Split

### Web

The web product is the network entry point. It is responsible for discovery, conversion, public proof, and early revenue.

Primary responsibilities:

- Let visitors understand the Agent Build Network.
- Show mock agents, projects, and cases before live marketplace mechanics exist.
- Give users a clear task submission path.
- Explain Desktop Node as a high-trust local capability.

### Desktop

The desktop product is named Cozmio Desktop Node.

Primary responsibilities:

- Observe real local work context.
- Preserve project history and execution results.
- Connect to execution tools such as Claude Code, Codex, Gemini, and custom CLIs.
- Register local capabilities into the web network, initially through mock/manual data.
- Produce delivery records that can later become cases or capability proof.

## Naming

Use Cozmio as the public brand across the redesigned web experience.

Do not use Pulseclaw as the visible homepage brand. Existing Pulseclaw material may remain in older pages or blog/history areas, but it should not define the new primary navigation, homepage, or conversion language.

## Homepage Copy

Hero title:

> 带着你的 Agent，帮别人把东西做出来。

Hero subtitle:

> Cozmio 让 Agent、Builder、项目和任务互相发现、协作、交付。桌面端会把你的电脑变成一个本地节点，连接真实项目、长期记忆和执行端。

Primary CTA:

> 提交任务

Secondary CTA:

> 查看桌面节点

Short supporting line:

> Agent 负责执行，Cozmio 负责接力、记录和交付。

## Navigation

Replace the current product-explainer navigation with network-first navigation:

- Agents
- Projects
- Cases
- Desktop
- Request

The primary header action should be:

- Submit a Task

If the page language is Chinese-first, use:

- 提交任务

## Pages

### `/`

The homepage should explain only the first useful layer:

1. This is an Agent Build Network.
2. Users can submit tasks.
3. Agents and builders can be discovered.
4. Desktop Node turns a user's computer into a local execution node.

Sections:

- Hero with title, subtitle, CTAs, and Agent Relay Network visual.
- Agent Network preview with four mock agent cards.
- Desktop Node section explaining local node capability.
- Cases preview with one real/self-referential case.
- Request CTA or embedded compact request form.

### `/agents`

Show mock agent cards.

Each card should include:

- Name
- Capabilities
- Task types
- Desktop Node connection status
- Case count or example case
- Status: Available, Busy, or Experimental

Initial examples:

- Cozmio Desktop Agent
- Claude Code Executor
- Research Agent
- Landing Page Agent

### `/projects`

Show early project/request listings using mock data.

Each project card should include:

- Project name
- Requested outcome
- Needed agent or builder type
- Budget/status if available
- Public case permission status

### `/cases`

Show delivery cases.

The first case should be:

> Cozmio 如何用桌面节点推进自己的 landing page 改版

Case structure:

- Task
- Agents or executors used
- Process
- Result
- Next step

### `/desktop`

Explain Cozmio Desktop Node.

Page title:

> 把你的电脑变成 Agent 网络里的本地节点。

Content should focus on:

- Connecting real local projects.
- Remembering execution history.
- Connecting Claude Code, Codex, Gemini, and custom CLIs.
- Turning local delivery results into cases or capability records.

Avoid making this page a generic desktop AI assistant page.

### `/request`

This is the most important conversion page.

The first version may be a static/mock form, but it must visibly collect:

- What the user wants to build.
- Budget or free trial preference.
- Needed agent or builder type.
- Whether public case publication is allowed.
- Contact information.

## Visual Direction

Do not use a 3D robot as the homepage's central metaphor.

Use an Agent Relay Network visual:

- Task Core or Build Request at the center.
- Agent nodes around it.
- Flow lines or moving tokens between nodes.
- A Memory/Cases sink showing delivered work becoming reusable proof.
- Status badges for availability, execution, and case history.

The visual should communicate:

- Task flow.
- Node collaboration.
- Execution status.
- Result delivery.
- Case/proof accumulation.

Preferred visual elements:

- Agent nodes.
- Task cards.
- Delivery case cards.
- Network lines.
- Status badges.
- Floating interface panels.

Avoid:

- Humanoid robots as the main visual.
- Generic brain/neural-network cliches.
- Internal architecture diagrams.
- Heavy 3D scenes that slow down first implementation.
- Purely decorative gradients or visual noise.

## First-Version Data Strategy

Use mock data for the first version.

The goal is not to implement full marketplace mechanics yet. The goal is to make the network understandable, navigable, and conversion-ready.

Mock data should be centralized so it can later be replaced by real data.

Recommended data groups:

- Agents
- Projects
- Cases
- Desktop capabilities
- Request form options

## Out of Scope

Do not build these in the first version:

- Real decentralized networking.
- Payment system.
- Full account system.
- Complete agent execution sandbox.
- P2P node communication.
- Complex reputation system.
- AI automatic task matching.
- Large dynamic 3D world.

## Success Criteria

The redesign is successful when:

- The homepage no longer reads as a context-system or desktop-assistant website.
- The visitor can tell that Cozmio is an Agent Build Network within the first screen.
- The navigation exposes Agents, Projects, Cases, Desktop, and Request.
- `/request` exists and provides a clear path to submit work.
- `/desktop` explains the local node difference without taking over the entire product story.
- The main visual language is nodes, tasks, flow, and cases, not robots.
- All first-version pages exist and can be navigated.

## Implementation Notes

The current web app is a Next.js site under `web/`.

Expected implementation areas:

- `web/src/lib/site-config.ts`
- `web/src/app/page.tsx`
- `web/src/components/HeroSection.tsx`
- `web/src/components/layout/Header.tsx`
- `web/src/components/layout/Footer.tsx`
- New route pages under `web/src/app/agents`, `web/src/app/projects`, `web/src/app/cases`, `web/src/app/desktop`, and `web/src/app/request`
- New shared mock data under `web/src/lib/`

Existing `/demo`, `/progress`, `/blog`, `/about`, and `/contact` routes may remain, but they should not be the primary navigation for the redesigned product.
