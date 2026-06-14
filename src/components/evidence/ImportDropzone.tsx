import { ActionButton } from "../ui/ActionButton";

export function ImportDropzone() {
  return (
    <section className="rounded-xl border border-dashed border-primary-soft/50 bg-panel/60 p-6 text-center">
      <p className="technical text-xs uppercase tracking-[0.05em] text-cyan">Read-only import area</p>
      <p className="mt-3 font-display text-lg font-semibold text-ink">Drop files for local metadata triage</p>
      <p className="mt-2 text-sm leading-6 text-muted">Static shell only. File picker and drag-and-drop processing are intentionally disabled.</p>
      <div className="mt-5">
        <ActionButton disabled variant="technical">
          Select files
        </ActionButton>
      </div>
    </section>
  );
}
