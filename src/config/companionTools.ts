import type { AppId } from "@/lib/api/types";

export type CompanionToolId = "ponytail" | "rtk" | "headroom";

export const COMPANION_TOOL_IDS: CompanionToolId[] = [
  "ponytail",
  "rtk",
  "headroom",
];

export const COMPANION_APP_IDS: AppId[] = [
  "claude",
  "codex",
  "gemini",
  "opencode",
];

export const COMPANION_TOOL_HOMEPAGES: Record<CompanionToolId, string> = {
  ponytail: "https://github.com/DietrichGebert/ponytail",
  rtk: "https://github.com/rtk-ai/rtk",
  headroom: "https://github.com/headroomlabs-ai/headroom",
};
