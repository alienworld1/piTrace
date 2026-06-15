import { ActionButton } from "../ui/ActionButton";

export function ReportActions() {
  return (
    <section className="panel-edge rounded-xl p-5">
      <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Export actions</p>
      <div className="mt-4 flex flex-wrap gap-3">
        <ActionButton disabled variant="technical">
          Export HTML
        </ActionButton>
        <ActionButton disabled variant="technical">
          Export JSON
        </ActionButton>
        <ActionButton disabled variant="technical">
          Export PDF
        </ActionButton>
      </div>
    </section>
  );
}
