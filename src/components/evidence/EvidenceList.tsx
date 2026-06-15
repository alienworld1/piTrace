import { Link } from "react-router";
import type { EvidenceFile } from "../../types/forensics";
import { formatBytes } from "../../utils/format";
import { Badge } from "../ui/Badge";

interface EvidenceListProps {
  files: EvidenceFile[];
}

export function EvidenceList({ files }: EvidenceListProps) {
  return (
    <section className="panel-edge rounded-xl p-5">
      <div className="flex items-center justify-between">
        <h2 className="font-display text-xl font-semibold text-ink">Evidence files</h2>
        <Badge tone="neutral">{files.length} items</Badge>
      </div>
      <div className="mt-5 space-y-3">
        {files.length === 0 ? (
          <p className="rounded-lg border border-line bg-surface px-4 py-5 text-sm text-muted">No evidence files imported yet.</p>
        ) : null}
        {files.map((file) => (
          <Link
            className="block rounded-lg border border-line bg-surface px-4 py-3 transition hover:border-cyan/50 hover:bg-panel-high"
            key={file.id}
            to={`/cases/${file.caseId}/files/${file.id}`}
          >
            <div className="flex items-center justify-between gap-3">
              <div className="min-w-0">
                <p className="truncate text-sm font-semibold text-ink">{file.fileName}</p>
                <p className="mt-1 text-xs text-muted">
                  {file.extension.toUpperCase()} · {formatBytes(file.sizeBytes)}
                </p>
              </div>
              <Badge tone={file.status}>{file.status}</Badge>
            </div>
            {file.errorMessage ? <p className="mt-2 text-xs text-danger">{file.errorMessage}</p> : null}
          </Link>
        ))}
      </div>
    </section>
  );
}
