import type { ReactNode } from "react";
import { Link } from "react-router";

interface ActionButtonProps {
  children: ReactNode;
  to?: string;
  variant?: "primary" | "secondary" | "technical";
  disabled?: boolean;
}

const variantClasses = {
  primary: "border-cyan bg-cyan text-[#0e0e0f] hover:bg-primary",
  secondary: "border-line bg-transparent text-ink hover:border-primary-soft hover:bg-panel-high",
  technical: "border-line bg-panel-high text-muted technical text-xs uppercase tracking-[0.05em] hover:border-cyan hover:text-cyan",
};

export function ActionButton({ children, to, variant = "secondary", disabled = false }: ActionButtonProps) {
  const className = `inline-flex min-h-10 items-center justify-center rounded-lg border px-4 text-sm font-semibold transition ${variantClasses[variant]} ${disabled ? "opacity-45" : ""}`;

  if (to && !disabled) {
    return (
      <Link className={className} to={to}>
        {children}
      </Link>
    );
  }

  return (
    <button className={className} disabled={disabled} type="button">
      {children}
    </button>
  );
}
