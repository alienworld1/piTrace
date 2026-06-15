import { ActionButton } from "../ui/ActionButton";

export function TopBar() {
  return (
    <header className="flex h-20 shrink-0 items-center justify-between border-b border-line bg-surface/80 px-8 backdrop-blur">
      <div>
        <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Desktop shell</p>
        <h1 className="font-display text-2xl font-semibold text-ink">Forensic metadata triage</h1>
      </div>
      <div className="flex items-center gap-3">
        <ActionButton to="/cases/new" variant="primary">
          New case
        </ActionButton>
      </div>
    </header>
  );
}
