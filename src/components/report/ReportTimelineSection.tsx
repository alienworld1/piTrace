import type { ReportTimelineEntry } from "../../types/forensics";

interface ReportTimelineSectionProps {
  entries: ReportTimelineEntry[];
}

export function ReportTimelineSection({ entries }: ReportTimelineSectionProps) {
  if (entries.length === 0) {
    return <p className="rounded-lg border border-line bg-surface px-4 py-5 text-sm text-muted">No normalized timeline metadata is available for this report.</p>;
  }

  return (
    <div className="overflow-x-auto rounded-lg border border-line">
      <table className="min-w-[720px] w-full text-left text-sm">
        <thead className="bg-base text-xs uppercase tracking-[0.05em] text-primary-soft">
          <tr>
            <th className="px-4 py-3 font-semibold">File</th>
            <th className="px-4 py-3 font-semibold">Field</th>
            <th className="px-4 py-3 font-semibold">Value</th>
            <th className="px-4 py-3 font-semibold">Source</th>
          </tr>
        </thead>
        <tbody>
          {entries.map((entry, index) => (
            <tr className="border-t border-line align-top" key={`${entry.fileId}:${entry.fieldLabel}:${index}`}>
              <td className="px-4 py-3 font-semibold text-ink">{entry.fileName}</td>
              <td className="px-4 py-3 text-muted">{entry.fieldLabel}</td>
              <td className="break-words px-4 py-3 text-muted">{entry.value}</td>
              <td className="px-4 py-3 technical text-xs text-cyan">{entry.source}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
