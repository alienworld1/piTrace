import type { MetadataCategory, MetadataField } from "../../types/forensics";
import { Badge } from "../ui/Badge";

interface MetadataCategoryGridProps {
  fields: MetadataField[];
}

type VisibleCategoryId = Extract<MetadataCategory, "identity" | "location" | "timeline" | "software" | "technical">;

interface VisibleCategory {
  id: VisibleCategoryId;
  title: string;
  emptyText: string;
}

const visibleCategories: VisibleCategory[] = [
  {
    id: "identity",
    title: "Identity",
    emptyText: "No author, owner, user, or organization metadata was identified.",
  },
  {
    id: "location",
    title: "Location",
    emptyText: "No GPS or place metadata was identified.",
  },
  {
    id: "timeline",
    title: "Timeline",
    emptyText: "No creation, modification, or media timeline metadata was identified.",
  },
  {
    id: "software",
    title: "Software/device",
    emptyText: "No software, encoder, camera, or device metadata was identified.",
  },
  {
    id: "technical",
    title: "Technical",
    emptyText: "No file format, dimensions, duration, or document structure metadata was identified.",
  },
];

const visibleCategoryIds = new Set<MetadataCategory>(visibleCategories.map((category) => category.id));

function isVisibleCategory(category: MetadataCategory | undefined): category is VisibleCategoryId {
  return Boolean(category && visibleCategoryIds.has(category));
}

export function MetadataCategoryGrid({ fields }: MetadataCategoryGridProps) {
  const fieldsByCategory = visibleCategories.reduce(
    (buckets, category) => {
      buckets[category.id] = [];
      return buckets;
    },
    {} as Record<VisibleCategoryId, MetadataField[]>,
  );

  let visibleFieldCount = 0;
  for (const field of fields) {
    if (!isVisibleCategory(field.normalizedCategory)) {
      continue;
    }

    fieldsByCategory[field.normalizedCategory].push(field);
    visibleFieldCount += 1;
  }

  return (
    <section className="panel-edge rounded-xl p-5">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="font-display text-xl font-semibold text-ink">Grouped metadata</h2>
          <p className="mt-1 text-sm text-muted">Readable fields derived from ExifTool metadata.</p>
        </div>
        <Badge tone={visibleFieldCount > 0 ? "primary" : "neutral"}>{visibleFieldCount} fields</Badge>
      </div>
      <div className="mt-5 grid gap-4 xl:grid-cols-2">
        {visibleCategories.map((category) => {
          const categoryFields = fieldsByCategory[category.id];

          return (
            <section className="rounded-lg border border-line bg-surface p-4" key={category.id}>
              <div className="flex items-center justify-between gap-3">
                <h3 className="font-display text-base font-semibold text-ink">{category.title}</h3>
                <span className="technical text-xs text-primary-soft">{categoryFields.length}</span>
              </div>
              {categoryFields.length === 0 ? (
                <p className="mt-4 text-sm leading-6 text-muted">{category.emptyText}</p>
              ) : (
                <dl className="mt-4 divide-y divide-line/80">
                  {categoryFields.map((field) => (
                    <div className="grid gap-2 py-3 sm:grid-cols-[170px_1fr]" key={field.id}>
                      <dt className="text-sm font-medium text-muted">{field.displayLabel ?? field.key}</dt>
                      <dd className="min-w-0 break-words text-sm text-ink">{field.value}</dd>
                    </div>
                  ))}
                </dl>
              )}
            </section>
          );
        })}
      </div>
    </section>
  );
}
