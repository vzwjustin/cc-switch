import type { Plugin } from "@opencode-ai/plugin"

// CC Switch RTK plugin — rewrites bash commands via `rtk rewrite`.
// Requires: rtk >= 0.23.0 on PATH.

const CcSwitchRtkPlugin: Plugin = async ({ $ }) => {
  try {
    await $`which rtk`.quiet()
  } catch {
    console.warn("[cc-switch-rtk] rtk binary not found in PATH — plugin disabled")
    return {}
  }

  return {
    "tool.execute.before": async (input, output) => {
      const tool = String(input?.tool ?? "").toLowerCase()
      if (tool !== "bash" && tool !== "shell") return
      const args = output?.args
      if (!args || typeof args !== "object") return

      const command = (args as Record<string, unknown>).command
      if (typeof command !== "string" || !command) return

      try {
        const result = await $`rtk rewrite ${command}`.quiet().nothrow()
        const rewritten = String(result.stdout).trim()
        if (rewritten && rewritten !== command) {
          ;(args as Record<string, unknown>).command = rewritten
        }
      } catch {
        // pass through unchanged
      }
    },
  }
}

export default CcSwitchRtkPlugin
