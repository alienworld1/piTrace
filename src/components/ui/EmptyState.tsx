interface EmptyStateProps {
  title: string;
  description: string;
}

export function EmptyState({ title, description }: EmptyStateProps) {
  return (
    <section className="panel-edge flex min-h-64 flex-col items-center justify-center rounded-xl px-8 py-12 text-center">
      <div className="flex h-12 w-12 items-center justify-center rounded-xl border border-cyan/30 bg-cyan/10 technical text-cyan">
        PT
      </div>
      <h2 className="mt-5 font-display text-2xl font-semibold text-ink">{title}</h2>
      <p className="mt-2 max-w-xl text-sm leading-6 text-muted">{description}</p>
    </section>
  );
}
