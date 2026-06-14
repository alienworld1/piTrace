import type { ReactNode } from "react";
import type { Confidence, EvidenceStatus, Severity } from "../../types/forensics";

type BadgeTone = Severity | Confidence | EvidenceStatus | "neutral" | "primary";

interface BadgeProps {
  children: ReactNode;
  tone?: BadgeTone;
}

const toneClasses: Record<BadgeTone, string> = {
  high: "border-danger/50 bg-danger-strong/25 text-danger",
  medium: "border-amber/50 bg-amber/10 text-amber",
  low: "border-primary-soft/50 bg-primary-soft/10 text-muted",
  pending: "border-primary-soft/50 bg-primary-soft/10 text-muted",
  hashing: "border-cyan/40 bg-cyan/10 text-cyan",
  analyzing: "border-cyan/40 bg-cyan/10 text-cyan",
  complete: "border-success/45 bg-success/10 text-success",
  error: "border-danger/50 bg-danger-strong/25 text-danger",
  neutral: "border-line bg-panel-high text-muted",
  primary: "border-cyan/50 bg-cyan/10 text-cyan",
};

export function Badge({ children, tone = "neutral" }: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center rounded-md border px-2 py-1 text-[11px] font-semibold uppercase leading-none tracking-[0.05em] ${toneClasses[tone]}`}
    >
      {children}
    </span>
  );
}
