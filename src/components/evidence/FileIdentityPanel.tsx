import type { ReactNode } from "react";
import type { EvidenceFile } from "../../types/forensics";
import { formatBytes, formatDateTime } from "../../utils/format";
import { Badge } from "../ui/Badge";

interface FileIdentityPanelProps {
  file: EvidenceFile;
  action?: ReactNode;
}

export function FileIdentityPanel({ file, action }: FileIdentityPanelProps) {
  return (
    <section className="panel-edge rounded-xl p-5">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">File identity</p>
          <h2 className="mt-2 font-display text-3xl font-semibold text-ink">{file.fileName}</h2>
          <p className="mt-2 technical text-xs text-muted">{file.originalPath}</p>
        </div>
        <div className="flex shrink-0 items-center gap-3">
          <Badge tone={file.status}>{file.status}</Badge>
          {action}
        </div>
      </div>
      <dl className="mt-6 grid grid-cols-4 gap-4 text-sm">
        <div>
          <dt className="text-muted">Type</dt>
          <dd className="mt-1 text-ink">{file.detectedFileType ?? "Pending"}</dd>
        </div>
        <div>
          <dt className="text-muted">MIME</dt>
          <dd className="mt-1 text-ink">{file.detectedMimeType ?? "Pending"}</dd>
        </div>
        <div>
          <dt className="text-muted">Size</dt>
          <dd className="mt-1 text-ink">{formatBytes(file.sizeBytes)}</dd>
        </div>
        <div>
          <dt className="text-muted">Imported</dt>
          <dd className="mt-1 text-ink">{formatDateTime(file.importedAt)}</dd>
        </div>
      </dl>
      {file.errorMessage ? (
        <div className="mt-4 rounded-lg border border-danger/40 bg-danger-strong/20 p-4 text-sm text-danger">{file.errorMessage}</div>
      ) : null}
    </section>
  );
}
