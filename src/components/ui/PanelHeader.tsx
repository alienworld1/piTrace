import type { ReactNode } from "react";

interface PanelHeaderProps {
  title: string;
  eyebrow?: string;
  action?: ReactNode;
}

export function PanelHeader({ title, eyebrow, action }: PanelHeaderProps) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div>
        {eyebrow ? <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">{eyebrow}</p> : null}
        <h2 className="mt-1 font-display text-xl font-semibold text-ink">{title}</h2>
      </div>
      {action ? <div className="shrink-0">{action}</div> : null}
    </div>
  );
}
