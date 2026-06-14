import type { EvidenceFile } from "../../types/forensics";
import { formatBytes, formatDateTime, shortHash } from "../../utils/format";
import { Badge } from "../ui/Badge";

interface FileIdentityPanelProps {
  file: EvidenceFile;
}

export function FileIdentityPanel({ file }: FileIdentityPanelProps) {
  return (
    <section className="panel-edge rounded-xl p-5">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">File identity</p>
          <h2 className="mt-2 font-display text-3xl font-semibold text-ink">{file.fileName}</h2>
          <p className="mt-2 technical text-xs text-muted">{file.originalPath}</p>
        </div>
        <Badge tone={file.status}>{file.status}</Badge>
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
      <div className="mt-6 rounded-lg border border-line bg-base p-4">
        <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Hashes</p>
        <p className="mt-3 technical text-sm text-ink">SHA-256 {shortHash(file.sha256)}</p>
        <p className="mt-2 technical text-sm text-muted">MD5 {shortHash(file.md5)}</p>
      </div>
    </section>
  );
}
