interface MetricCardProps {
  label: string;
  value: string;
  detail?: string;
}

export function MetricCard({ label, value, detail }: MetricCardProps) {
  return (
    <section className="panel-edge rounded-xl p-4">
      <p className="text-xs font-semibold uppercase tracking-[0.05em] text-muted">{label}</p>
      <p className="mt-3 font-display text-3xl font-semibold leading-none text-ink">{value}</p>
      {detail ? <p className="mt-2 text-sm text-muted">{detail}</p> : null}
    </section>
  );
}
