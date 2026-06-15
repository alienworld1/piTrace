import { Link } from "react-router";
import type { EvidenceFile } from "../../types/forensics";
import { formatBytes } from "../../utils/format";
import { Badge } from "../ui/Badge";

interface EvidenceListProps {
  files: EvidenceFile[];
  onRemoveFile: (file: EvidenceFile) => void | Promise<void>;
}

export function EvidenceList({ files, onRemoveFile }: EvidenceListProps) {
  return (
    <section className="panel-edge rounded-xl p-5">
      <div className="flex items-center justify-between">
        <h2 className="font-display text-xl font-semibold text-ink">Evidence files</h2>
        <Badge tone="neutral">{files.length} items</Badge>
      </div>
      <div className="mt-5 space-y-3">
        {files.length === 0 ? (
          <p className="rounded-lg border border-line bg-surface px-4 py-5 text-sm text-muted">No evidence files imported yet. Drop files above or use Select files.</p>
        ) : null}
        {files.map((file) => (
          <div className="rounded-lg border border-line bg-surface px-4 py-3 transition hover:border-cyan/50 hover:bg-panel-high" key={file.id}>
            <div className="flex items-start justify-between gap-3">
              <Link className="min-w-0 flex-1" to={`/cases/${file.caseId}/files/${file.id}`}>
                <p className="truncate text-sm font-semibold text-ink">{file.fileName}</p>
                <p className="mt-1 truncate text-xs text-muted">{parentPath(file.originalPath)}</p>
                <p className="mt-1 text-xs text-muted">
                  {(file.detectedFileType ?? file.extension.toUpperCase())} · {formatBytes(file.sizeBytes)}
                </p>
              </Link>
              <div className="flex shrink-0 items-center gap-2">
                <Badge tone={file.status}>{file.status}</Badge>
                <button
                  className="rounded-md border border-danger/40 px-3 py-1.5 text-xs font-semibold text-danger transition hover:border-danger hover:bg-danger-strong/20"
                  onClick={() => void onRemoveFile(file)}
                  type="button"
                >
                  Remove
                </button>
              </div>
            </div>
            {file.errorMessage ? <p className="mt-2 text-xs text-danger">{file.errorMessage}</p> : null}
          </div>
        ))}
      </div>
    </section>
  );
}

function parentPath(path: string) {
  const normalized = path.replace(/\\/g, "/");
  const index = normalized.lastIndexOf("/");
  if (index <= 0) {
    return path;
  }

  return normalized.slice(0, index);
}
