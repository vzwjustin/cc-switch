import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  settingsApi,
  type AgentToolsConfig,
  type AgentToolsStatus,
} from "@/lib/api/settings";

export function AgentToolsConfigPanel() {
  const { t } = useTranslation();
  const [config, setConfig] = useState<AgentToolsConfig>({
    headroomEnabled: false,
    headroomPort: 8787,
    headroomAutoStart: false,
    rtkEnabled: false,
    rtkClaude: true,
    rtkCodex: true,
    ponytailEnabled: false,
    ponytailMode: "full",
    ponytailInstallSkill: false,
  });
  const [status, setStatus] = useState<AgentToolsStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const refreshStatus = () => {
    settingsApi
      .getAgentToolsStatus()
      .then(setStatus)
      .catch((e) => console.error("Failed to load agent tools status:", e));
  };

  useEffect(() => {
    Promise.all([
      settingsApi.getAgentToolsConfig(),
      settingsApi.getAgentToolsStatus(),
    ])
      .then(([loadedConfig, loadedStatus]) => {
        setConfig(loadedConfig);
        setStatus(loadedStatus);
      })
      .catch((e) => console.error("Failed to load agent tools config:", e))
      .finally(() => setIsLoading(false));
  }, []);

  const handleChange = async (updates: Partial<AgentToolsConfig>) => {
    const newConfig = { ...config, ...updates };
    setConfig(newConfig);
    try {
      await settingsApi.setAgentToolsConfig(newConfig);
      refreshStatus();
    } catch (e) {
      console.error("Failed to save agent tools config:", e);
      toast.error(String(e));
      setConfig(config);
    }
  };

  if (isLoading) return null;

  return (
    <div className="space-y-8">
      {/* Headroom */}
      <section className="space-y-4">
        <div className="flex items-center justify-between gap-4">
          <div className="space-y-0.5">
            <Label>{t("settings.advanced.agentTools.headroom.enabled")}</Label>
            <p className="text-xs text-muted-foreground">
              {t("settings.advanced.agentTools.headroom.enabledDescription")}
            </p>
          </div>
          <Switch
            checked={config.headroomEnabled}
            onCheckedChange={(checked) =>
              handleChange({ headroomEnabled: checked })
            }
          />
        </div>

        <div className="flex flex-wrap items-center gap-2 pl-1">
          <Badge variant={status?.headroomInstalled ? "default" : "secondary"}>
            {status?.headroomInstalled
              ? t("settings.advanced.agentTools.installed")
              : t("settings.advanced.agentTools.notInstalled")}
          </Badge>
          {status?.headroomInstalled && status.headroomVersion && (
            <Badge variant="outline">{status.headroomVersion}</Badge>
          )}
          {config.headroomEnabled && (
            <Badge variant={status?.headroomRunning ? "default" : "secondary"}>
              {status?.headroomRunning
                ? t("settings.advanced.agentTools.running")
                : t("settings.advanced.agentTools.stopped")}
            </Badge>
          )}
        </div>

        <div className="grid gap-4 pl-4 sm:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="headroom-port">
              {t("settings.advanced.agentTools.headroom.port")}
            </Label>
            <Input
              id="headroom-port"
              type="number"
              min={1}
              max={65535}
              value={config.headroomPort}
              disabled={!config.headroomEnabled}
              onChange={(e) =>
                setConfig({
                  ...config,
                  headroomPort: Number(e.target.value) || 8787,
                })
              }
              onBlur={() => handleChange({ headroomPort: config.headroomPort })}
            />
          </div>
          <div className="flex items-center justify-between gap-4 pt-6">
            <div className="space-y-0.5">
              <Label>
                {t("settings.advanced.agentTools.headroom.autoStart")}
              </Label>
              <p className="text-xs text-muted-foreground">
                {t("settings.advanced.agentTools.headroom.autoStartDescription")}
              </p>
            </div>
            <Switch
              checked={config.headroomAutoStart}
              disabled={!config.headroomEnabled}
              onCheckedChange={(checked) =>
                handleChange({ headroomAutoStart: checked })
              }
            />
          </div>
        </div>
      </section>

      {/* RTK */}
      <section className="space-y-4 border-t border-border/50 pt-6">
        <div className="flex items-center justify-between gap-4">
          <div className="space-y-0.5">
            <Label>{t("settings.advanced.agentTools.rtk.enabled")}</Label>
            <p className="text-xs text-muted-foreground">
              {t("settings.advanced.agentTools.rtk.enabledDescription")}
            </p>
          </div>
          <Switch
            checked={config.rtkEnabled}
            onCheckedChange={(checked) => handleChange({ rtkEnabled: checked })}
          />
        </div>

        <div className="flex flex-wrap items-center gap-2 pl-1">
          <Badge variant={status?.rtkInstalled ? "default" : "secondary"}>
            {status?.rtkInstalled
              ? t("settings.advanced.agentTools.installed")
              : t("settings.advanced.agentTools.notInstalled")}
          </Badge>
          {status?.rtkInstalled && status.rtkVersion && (
            <Badge variant="outline">{status.rtkVersion}</Badge>
          )}
        </div>

        <div className="space-y-3 pl-4">
          <div className="flex items-center justify-between">
            <Label>{t("settings.advanced.agentTools.rtk.claude")}</Label>
            <Switch
              checked={config.rtkClaude}
              disabled={!config.rtkEnabled}
              onCheckedChange={(checked) =>
                handleChange({ rtkClaude: checked })
              }
            />
          </div>
          <div className="flex items-center justify-between">
            <Label>{t("settings.advanced.agentTools.rtk.codex")}</Label>
            <Switch
              checked={config.rtkCodex}
              disabled={!config.rtkEnabled}
              onCheckedChange={(checked) =>
                handleChange({ rtkCodex: checked })
              }
            />
          </div>
        </div>
      </section>

      {/* Ponytail */}
      <section className="space-y-4 border-t border-border/50 pt-6">
        <div className="flex items-center justify-between gap-4">
          <div className="space-y-0.5">
            <Label>{t("settings.advanced.agentTools.ponytail.enabled")}</Label>
            <p className="text-xs text-muted-foreground">
              {t("settings.advanced.agentTools.ponytail.enabledDescription")}
            </p>
          </div>
          <Switch
            checked={config.ponytailEnabled}
            onCheckedChange={(checked) =>
              handleChange({ ponytailEnabled: checked })
            }
          />
        </div>

        {status?.ponytailSkillInstalled && (
          <Badge variant="outline" className="ml-1">
            {t("settings.advanced.agentTools.ponytail.skillInstalled")}
          </Badge>
        )}

        <div className="grid gap-4 pl-4 sm:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="ponytail-mode">
              {t("settings.advanced.agentTools.ponytail.mode")}
            </Label>
            <select
              id="ponytail-mode"
              className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
              value={config.ponytailMode}
              disabled={!config.ponytailEnabled}
              onChange={(e) => handleChange({ ponytailMode: e.target.value })}
            >
              <option value="lite">
                {t("settings.advanced.agentTools.ponytail.modes.lite")}
              </option>
              <option value="full">
                {t("settings.advanced.agentTools.ponytail.modes.full")}
              </option>
              <option value="ultra">
                {t("settings.advanced.agentTools.ponytail.modes.ultra")}
              </option>
              <option value="off">
                {t("settings.advanced.agentTools.ponytail.modes.off")}
              </option>
            </select>
          </div>
          <div className="flex items-center justify-between gap-4 pt-6">
            <div className="space-y-0.5">
              <Label>
                {t("settings.advanced.agentTools.ponytail.installSkill")}
              </Label>
              <p className="text-xs text-muted-foreground">
                {t(
                  "settings.advanced.agentTools.ponytail.installSkillDescription",
                )}
              </p>
            </div>
            <Switch
              checked={config.ponytailInstallSkill}
              disabled={!config.ponytailEnabled}
              onCheckedChange={(checked) =>
                handleChange({ ponytailInstallSkill: checked })
              }
            />
          </div>
        </div>
      </section>
    </div>
  );
}
