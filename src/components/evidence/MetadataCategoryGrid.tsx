import type { MetadataField } from "../../types/forensics";

interface MetadataCategoryGridProps {
  fields: MetadataField[];
}

export function MetadataCategoryGrid({ fields }: MetadataCategoryGridProps) {
  const categories = Array.from(new Set(fields.map((field) => field.normalizedCategory ?? "other")));

  return (
    <section className="panel-edge rounded-xl p-5">
      <h2 className="font-display text-xl font-semibold text-ink">Metadata categories</h2>
      <div className="mt-5 grid grid-cols-3 gap-4">
        {categories.map((category) => {
          const categoryFields = fields.filter((field) => (field.normalizedCategory ?? "other") === category);

          return (
            <div className="rounded-lg border border-line bg-surface p-4" key={category}>
              <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">{category}</p>
              <p className="mt-3 font-display text-2xl font-semibold text-ink">{categoryFields.length}</p>
              <p className="mt-2 text-sm text-muted">fields identified</p>
            </div>
          );
        })}
      </div>
    </section>
  );
}
