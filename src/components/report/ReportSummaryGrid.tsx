import type { ReportSummary } from "../../types/forensics";

interface ReportSummaryGridProps {
  summary: ReportSummary;
}

export function ReportSummaryGrid({ summary }: ReportSummaryGridProps) {
  const metrics = [
    { label: "Evidence items", value: summary.evidenceCount, tone: "text-ink" },
    { label: "Findings", value: summary.findingCount, tone: "text-ink" },
    { label: "High severity", value: summary.highCount, tone: "text-danger" },
    { label: "Medium severity", value: summary.mediumCount, tone: "text-amber" },
    { label: "Low severity", value: summary.lowCount, tone: "text-success" },
    { label: "Complete files", value: summary.completeFileCount, tone: "text-success" },
    { label: "Pending files", value: summary.pendingFileCount, tone: "text-muted" },
    { label: "Error files", value: summary.errorFileCount, tone: "text-danger" },
  ];

  return (
    <div className="mt-8 grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
      {metrics.map((metric) => (
        <div className="rounded-lg border border-line bg-base p-4" key={metric.label}>
          <p className="text-sm text-muted">{metric.label}</p>
          <p className={`mt-2 technical text-2xl ${metric.tone}`}>{metric.value}</p>
        </div>
      ))}
    </div>
  );
}
