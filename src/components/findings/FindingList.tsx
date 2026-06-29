import { Link } from "react-router";
import type { Finding, Severity } from "../../types/forensics";
import { Badge } from "../ui/Badge";

interface FindingListProps {
  caseId: string;
  findings: Finding[];
}

const severityOrder: Severity[] = ["high", "medium", "low"];

export function FindingList({ caseId, findings }: FindingListProps) {
  const groupedFindings = severityOrder
    .map((severity) => ({
      severity,
      findings: findings
        .filter((finding) => finding.severity === severity)
        .sort((left, right) => {
          const category = left.category.localeCompare(right.category);
          if (category !== 0) {
            return category;
          }
          const title = left.title.localeCompare(right.title);
          if (title !== 0) {
            return title;
          }
          return right.createdAt.localeCompare(left.createdAt);
        }),
    }))
    .filter((group) => group.findings.length > 0);

  return (
    <section className="panel-edge rounded-xl p-5">
      <div className="flex items-center justify-between">
        <h2 className="font-display text-xl font-semibold text-ink">Findings</h2>
        <Badge tone="neutral">{findings.length} indicators</Badge>
      </div>
      <div className="mt-5 space-y-3">
        {findings.length === 0 ? (
          <p className="rounded-lg border border-line bg-surface px-4 py-5 text-sm text-muted">No findings have been generated yet.</p>
        ) : null}
        {groupedFindings.map((group) => (
          <div className="space-y-3" key={group.severity}>
            <div className="flex items-center justify-between border-b border-line/70 pb-2">
              <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">{group.severity}</p>
              <Badge tone={group.severity}>{group.findings.length}</Badge>
            </div>
            {group.findings.map((finding) => (
              <Link
                className="block rounded-lg border border-line bg-surface p-4 transition hover:border-cyan/50 hover:bg-panel-high"
                key={finding.id}
                to={`/cases/${caseId}/findings/${finding.id}`}
              >
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm font-semibold text-ink">{finding.title}</p>
                  <Badge tone={finding.severity}>{finding.severity}</Badge>
                </div>
                <p className="mt-2 text-sm leading-6 text-muted">{finding.description}</p>
                <div className="mt-3 flex flex-wrap gap-2">
                  <Badge tone="neutral">{finding.category}</Badge>
                  <Badge tone={finding.confidence}>{finding.confidence} confidence</Badge>
                </div>
              </Link>
            ))}
          </div>
        ))}
      </div>
    </section>
  );
}
