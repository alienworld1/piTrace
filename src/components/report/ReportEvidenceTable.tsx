import type { EvidenceFile } from "../../types/forensics";
import { formatBytes, formatDateTime } from "../../utils/format";
import { Badge } from "../ui/Badge";

interface ReportEvidenceTableProps {
  files: EvidenceFile[];
}

export function ReportEvidenceTable({ files }: ReportEvidenceTableProps) {
  if (files.length === 0) {
    return <p className="rounded-lg border border-line bg-surface px-4 py-5 text-sm text-muted">No evidence files are available for this report yet.</p>;
  }

  return (
    <div className="overflow-x-auto rounded-lg border border-line">
      <table className="min-w-[760px] w-full text-left text-sm">
        <thead className="bg-base text-xs uppercase tracking-[0.05em] text-primary-soft">
          <tr>
            <th className="px-4 py-3 font-semibold">File</th>
            <th className="px-4 py-3 font-semibold">Type</th>
            <th className="px-4 py-3 font-semibold">Imported</th>
            <th className="px-4 py-3 font-semibold">Hash</th>
            <th className="px-4 py-3 font-semibold">Status</th>
          </tr>
        </thead>
        <tbody>
          {files.map((file) => (
            <tr className="border-t border-line align-top" key={file.id}>
              <td className="px-4 py-3">
                <p className="font-semibold text-ink">{file.fileName}</p>
                <p className="mt-1 text-xs text-muted">{formatBytes(file.sizeBytes)}</p>
              </td>
              <td className="px-4 py-3 text-muted">{file.detectedFileType ?? "Unknown"}</td>
              <td className="px-4 py-3 text-muted">{formatDateTime(file.importedAt)}</td>
              <td className="break-all px-4 py-3 technical text-xs text-muted">{file.sha256 ?? "Not available"}</td>
              <td className="px-4 py-3">
                <Badge tone={file.status}>{file.status}</Badge>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
