import type { Finding } from "../../types/forensics";
import { formatDateTime } from "../../utils/format";
import { Badge } from "../ui/Badge";

interface ReportFindingsSectionProps {
  findings: Finding[];
}

export function ReportFindingsSection({ findings }: ReportFindingsSectionProps) {
  if (findings.length === 0) {
    return <p className="rounded-lg border border-line bg-surface px-4 py-5 text-sm text-muted">No rule-based findings are recorded for this case.</p>;
  }

  return (
    <div className="space-y-3">
      {findings.map((finding) => (
        <article className="rounded-lg border border-line bg-surface p-4" key={finding.id}>
          <div className="flex items-start justify-between gap-4">
            <div>
              <h3 className="font-display text-lg font-semibold text-ink">{finding.title}</h3>
              <p className="mt-2 text-sm leading-6 text-muted">{finding.description}</p>
            </div>
            <Badge tone={finding.severity}>{finding.severity}</Badge>
          </div>
          <p className="mt-3 text-xs uppercase tracking-[0.05em] text-primary-soft">
            {finding.category} · {finding.confidence} confidence · {formatDateTime(finding.createdAt)}
          </p>
        </article>
      ))}
    </div>
  );
}
