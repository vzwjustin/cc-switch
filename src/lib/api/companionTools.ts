import { invoke } from "@tauri-apps/api/core";
import type { AppId } from "./types";
import type { CompanionToolId } from "@/config/companionTools";

export interface CompanionToolAppStatus {
  app: AppId;
  installed: boolean;
}

export interface CompanionToolStatus {
  tool: CompanionToolId;
  apps: CompanionToolAppStatus[];
}

export const companionToolsApi = {
  async getStatus(): Promise<CompanionToolStatus[]> {
    return await invoke("get_companion_tools_status");
  },

  async install(tool: CompanionToolId, app: AppId): Promise<void> {
    await invoke("install_companion_tool", { tool, app });
  },

  async uninstall(tool: CompanionToolId, app: AppId): Promise<void> {
    await invoke("uninstall_companion_tool", { tool, app });
  },
};
