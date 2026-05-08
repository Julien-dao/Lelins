// Typed wrappers around `@tauri-apps/api/core::invoke` so the UI never
// passes free-form strings to the backend. Generated TypeScript types
// will replace this hand-written file once we wire up `ts-rs` in a later
// step; for now hand-typed structs match `apps/desktop/src-tauri/src/commands.rs`.

import { invoke } from "@tauri-apps/api/core";

export type ChatReply = { answer: string };
export type ChatHistoryEntry = { role: "user" | "assistant" | "system"; content: string };

export type LicenseSummary = {
  canonical: string;
  tier: "DECO" | "PRO" | "MAIT" | "BNDL";
  version: number;
  issued_days: number;
  features: number;
};

export type ModelChoice = {
  tier: "premium" | "standard" | "light";
  ollama_id: string;
  display_name: string;
  disk_size_gb: number;
  runtime_ram_gb: number;
  blurb: string;
};

export type HardwareProfile = {
  total_ram_gb: number;
  logical_cpus: number;
  cpu_brand: string | null;
  arch: string;
  os: string;
};

export type RagHit = {
  id: string;
  citation: string;
  ccp: string | null;
  cp: string | null;
  snippet: string;
  score: number;
};

export const ipc = {
  ping: () => invoke<string>("ping"),

  licenseValidate: (raw_key: string, email: string) =>
    invoke<LicenseSummary>("license_validate", { rawKey: raw_key, email }),

  licenseInfo: (raw_key: string) =>
    invoke<string>("license_info", { rawKey: raw_key }),

  hardwareProfile: () => invoke<HardwareProfile>("hardware_profile"),
  recommendedModel: () => invoke<ModelChoice>("recommended_model"),
  minimumRamGb: () => invoke<number>("minimum_ram_gb"),

  chatSendText: (message: string) =>
    invoke<ChatReply>("chat_send_text", { message }),
  chatHistory: () => invoke<ChatHistoryEntry[]>("chat_history"),
  chatReset: () => invoke<void>("chat_reset"),

  ragSearch: (query: string, top_k = 4) =>
    invoke<RagHit[]>("rag_search", { query, topK: top_k }),
  ragStatus: () => invoke<number>("rag_status"),
};
