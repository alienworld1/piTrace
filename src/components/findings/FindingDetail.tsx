import type { Finding, MetadataField } from "../../types/forensics";
import { Badge } from "../ui/Badge";

interface FindingDetailProps {
  finding: Finding;
  relatedFields: MetadataField[];
}

function whyItMatters(finding: Finding) {
  if (finding.title === "No embedded timestamp metadata found") {
    return "Without embedded timestamps, this file may provide less timeline context than other evidence items. Review filesystem dates and surrounding case material before drawing conclusions.";
  }

  switch (finding.category) {
    case "location":
      return "GPS coordinates can reveal where a file may have been captured or created. Treat this as sensitive location context and confirm it against the raw metadata values.";
    case "identity":
      return "Identity metadata can expose an author, owner, device user, or organization. These fields are useful for attribution leads but should be corroborated before reporting.";
    case "software":
      return "Software and device fields can show how a file was created, edited, encoded, or exported. This helps reconstruct workflow without proving intent by itself.";
    case "timeline":
      return "Timestamp metadata can support chronology, but conflicts and missing values are common. Compare these fields with file system dates and other evidence before relying on them.";
    case "integrity":
      return "A mismatch between declared extension and detected type may indicate renaming, conversion, or misleading packaging. Review the file identity before sharing or reporting it.";
    case "privacy":
      return "This score summarizes metadata that may reveal personal, location, device, organization, or software context. It is a triage aid only, not a definitive risk assessment.";
    default:
      return "Review the related metadata fields and raw values before relying on this indicator in a report.";
  }
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
      <div className="mt-6 grid gap-4 md:grid-cols-2">
        <div className="rounded-lg border border-line bg-base p-4">
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Why it matters</p>
          <p className="mt-3 text-sm leading-6 text-muted">{whyItMatters(finding)}</p>
        </div>
        <div className="rounded-lg border border-line bg-base p-4">
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Category</p>
          <p className="mt-3 text-sm capitalize text-ink">{finding.category}</p>
        </div>
      </div>
      <div className="mt-6 rounded-lg border border-line bg-surface p-4">
        <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Related fields</p>
        <div className="mt-4 space-y-3">
          {relatedFields.length === 0 ? (
            <p className="rounded-md bg-panel px-3 py-3 text-sm leading-6 text-muted">
              This finding is based on the absence of normalized supporting fields or on file-level context rather than a specific metadata value.
            </p>
          ) : null}
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
