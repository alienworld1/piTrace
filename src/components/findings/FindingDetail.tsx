import type { Finding, MetadataField } from "../../types/forensics";
import { Badge } from "../ui/Badge";

interface FindingDetailProps {
  finding: Finding;
  relatedFields: MetadataField[];
}

export function FindingDetail({ finding, relatedFields }: FindingDetailProps) {
  return (
    <section className="panel-edge rounded-xl p-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Finding detail</p>
          <h2 className="mt-2 font-display text-3xl font-semibold text-ink">{finding.title}</h2>
        </div>
        <div className="flex gap-2">
          <Badge tone={finding.severity}>{finding.severity}</Badge>
          <Badge tone={finding.confidence}>{finding.confidence}</Badge>
        </div>
      </div>
      <p className="mt-5 max-w-3xl text-sm leading-6 text-muted">{finding.description}</p>
      <div className="mt-6 grid grid-cols-2 gap-4">
        <div className="rounded-lg border border-line bg-base p-4">
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Why it matters</p>
          <p className="mt-3 text-sm leading-6 text-muted">
            This indicator may expose identity, location, device, or workflow context. It should be reviewed before the file is shared or used in a report.
          </p>
        </div>
        <div className="rounded-lg border border-line bg-base p-4">
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Category</p>
          <p className="mt-3 text-sm capitalize text-ink">{finding.category}</p>
        </div>
      </div>
      <div className="mt-6 rounded-lg border border-line bg-surface p-4">
        <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Related fields</p>
        <div className="mt-4 space-y-3">
          {relatedFields.map((field) => (
            <div className="grid gap-3 rounded-md bg-panel px-3 py-2 text-sm sm:grid-cols-[180px_1fr]" key={field.id}>
              <div className="min-w-0">
                <p className="break-words technical text-xs text-cyan">
                  {field.group}:{field.key}
                </p>
                {field.displayLabel && field.displayLabel !== field.key ? (
                  <p className="mt-1 text-xs text-muted">{field.displayLabel}</p>
                ) : null}
              </div>
              <p className="min-w-0 break-words text-muted">{field.value}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
