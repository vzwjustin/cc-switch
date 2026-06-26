import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface AppNavButtonProps {
  icon: ReactNode;
  label: string;
  active?: boolean;
  onClick: () => void;
  compact?: boolean;
  hidden?: boolean;
  title?: string;
}

export function AppNavButton({
  icon,
  label,
  active = false,
  onClick,
  compact = false,
  hidden = false,
  title,
}: AppNavButtonProps) {
  if (hidden) {
    return null;
  }

  return (
    <button
      type="button"
      onClick={onClick}
      title={title ?? label}
      aria-label={label}
      aria-current={active ? "page" : undefined}
      className={cn(
        "inline-flex items-center justify-center gap-1.5 rounded-lg text-sm font-medium transition-all duration-200",
        "text-muted-foreground hover:text-foreground hover:bg-accent/80",
        compact ? "h-8 w-8 px-0" : "h-8 px-2.5",
        active && "bg-background text-foreground shadow-sm ring-1 ring-border/60",
      )}
    >
      <span className="inline-flex shrink-0 items-center justify-center">{icon}</span>
      {!compact && (
        <span className="hidden whitespace-nowrap text-xs font-medium sm:inline">
          {label}
        </span>
      )}
    </button>
  );
}
