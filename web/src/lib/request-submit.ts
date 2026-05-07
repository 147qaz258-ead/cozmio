const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "/api";

export type CozmioRequestPayload = {
  task: string;
  projectType: string;
  budget: string;
  agentType: string;
  timeline: string;
  publicCase: boolean;
  contact: string;
};

export async function submitCozmioRequest(payload: CozmioRequestPayload) {
  const res = await fetch(`${API_BASE}/applications`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      name: payload.contact.split("@")[0] || payload.contact,
      email: payload.contact,
      useCase: payload.task,
      role: payload.projectType,
      source: `website_${payload.agentType}`,
    }),
  });

  const data = await res.json();
  if (!res.ok || !data.ok) {
    throw new Error(data.error?.message || "提交失败");
  }
  return data.data;
}
