import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { ExternalLink, Gauge, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AppToggleGroup } from "@/components/common/AppToggleGroup";
import {
  COMPANION_APP_IDS,
  COMPANION_TOOL_HOMEPAGES,
  COMPANION_TOOL_IDS,
  type CompanionToolId,
} from "@/config/companionTools";
import type { AppId } from "@/lib/api/types";
import {
  companionToolStatusMap,
  useCompanionToolsStatus,
  useInstallCompanionTool,
  useUninstallCompanionTool,
} from "@/hooks/useCompanionTools";
import { extractErrorMessage } from "@/utils/errorUtils";

interface CompanionToolsPanelProps {
  onOpenChange: (open: boolean) => void;
}

const CompanionToolsPanel: React.FC<CompanionToolsPanelProps> = ({
  onOpenChange: _onOpenChange,
}) => {
  const { t } = useTranslation();
  const { data: statuses, isLoading, refetch } = useCompanionToolsStatus();
  const installMutation = useInstallCompanionTool();
  const uninstallMutation = useUninstallCompanionTool();
  const [pendingKey, setPendingKey] = useState<string | null>(null);

  const statusMap = useMemo(() => companionToolStatusMap(statuses), [statuses]);

  const handleToggleApp = async (
    tool: CompanionToolId,
    app: AppId,
    enabled: boolean,
  ) => {
    const key = `${tool}:${app}`;
    setPendingKey(key);
    try {
      if (enabled) {
        await installMutation.mutateAsync({ tool, app });
        toast.success(t("companionTools.installSuccess", { tool, app }));
      } else {
        await uninstallMutation.mutateAsync({ tool, app });
        toast.success(t("companionTools.uninstallSuccess", { tool, app }));
      }
      await refetch();
    } catch (error) {
      toast.error(t("common.error"), {
        description: extractErrorMessage(error),
      });
    } finally {
      setPendingKey(null);
    }
  };

  if (isLoading) {
    return (
      <div className="px-6 pt-4 pb-8 flex items-center justify-center min-h-[200px]">
        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <TooltipProvider>
      <div className="px-6 pt-4 pb-8 space-y-4">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <Gauge className="h-5 w-5 text-primary" />
            <h2 className="text-lg font-semibold">{t("companionTools.title")}</h2>
          </div>
          <p className="text-sm text-muted-foreground max-w-3xl">
            {t("companionTools.description")}
          </p>
        </div>

        <div className="grid gap-3">
          {COMPANION_TOOL_IDS.map((tool) => {
            const appStates = COMPANION_APP_IDS.reduce(
              (acc, app) => {
                acc[app] = statusMap.get(tool)?.get(app) ?? false;
                return acc;
              },
              {} as Partial<Record<AppId, boolean>>,
            );

            return (
              <div
                key={tool}
                className="rounded-xl border border-border bg-card/60 p-4 shadow-sm"
              >
                <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                  <div className="min-w-0 space-y-1">
                    <div className="flex items-center gap-2">
                      <h3 className="text-sm font-medium">
                        {t(`companionTools.tools.${tool}.name`)}
                      </h3>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7"
                        asChild
                      >
                        <a
                          href={COMPANION_TOOL_HOMEPAGES[tool]}
                          target="_blank"
                          rel="noreferrer"
                          title={t("companionTools.homepage")}
                        >
                          <ExternalLink className="h-3.5 w-3.5" />
                        </a>
                      </Button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {t(`companionTools.tools.${tool}.description`)}
                    </p>
                  </div>

                  <div className="flex items-center gap-2">
                    {pendingKey?.startsWith(`${tool}:`) && (
                      <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                    )}
                    <AppToggleGroup
                      appIds={COMPANION_APP_IDS}
                      apps={appStates}
                      onToggle={(app, enabled) =>
                        void handleToggleApp(tool, app, enabled)
                      }
                    />
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </TooltipProvider>
  );
};

export default CompanionToolsPanel;
