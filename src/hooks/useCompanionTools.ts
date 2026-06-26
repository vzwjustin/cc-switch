import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  companionToolsApi,
  type CompanionToolStatus,
} from "@/lib/api/companionTools";
import type { AppId } from "@/lib/api/types";
import type { CompanionToolId } from "@/config/companionTools";

export const companionToolsKeys = {
  all: ["companionTools"] as const,
  status: () => [...companionToolsKeys.all, "status"] as const,
};

export function useCompanionToolsStatus() {
  return useQuery({
    queryKey: companionToolsKeys.status(),
    queryFn: () => companionToolsApi.getStatus(),
  });
}

export function useInstallCompanionTool() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ tool, app }: { tool: CompanionToolId; app: AppId }) =>
      companionToolsApi.install(tool, app),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: companionToolsKeys.all });
    },
  });
}

export function useUninstallCompanionTool() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ tool, app }: { tool: CompanionToolId; app: AppId }) =>
      companionToolsApi.uninstall(tool, app),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: companionToolsKeys.all });
    },
  });
}

export function companionToolStatusMap(
  statuses: CompanionToolStatus[] | undefined,
): Map<CompanionToolId, Map<AppId, boolean>> {
  const map = new Map<CompanionToolId, Map<AppId, boolean>>();
  for (const entry of statuses ?? []) {
    const appMap = new Map<AppId, boolean>();
    for (const appStatus of entry.apps) {
      appMap.set(appStatus.app, appStatus.installed);
    }
    map.set(entry.tool, appMap);
  }
  return map;
}
