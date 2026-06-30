import { Link } from "react-router";
import { useState } from "react";
import type { EvidenceFile } from "../../types/forensics";
import { formatBytes } from "../../utils/format";
import { Badge } from "../ui/Badge";
import { ErrorNotice } from "../ui/ErrorNotice";

interface EvidenceListProps {
  deletingFileId?: string;
  files: EvidenceFile[];
  isRemoveDisabled?: boolean;
  onRemoveFile: (file: EvidenceFile) => void | Promise<void>;
}

export function EvidenceList({ deletingFileId, files, isRemoveDisabled = false, onRemoveFile }: EvidenceListProps) {
  const [visiblePathIds, setVisiblePathIds] = useState<Set<string>>(new Set());

  function togglePath(fileId: string) {
    setVisiblePathIds((current) => {
      const next = new Set(current);
      if (next.has(fileId)) {
        next.delete(fileId);
      } else {
        next.add(fileId);
      }
      return next;
    });
  }

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
                <p className="mt-1 truncate text-xs text-muted">Folder: {parentFolder(file.originalPath)}</p>
                <p className="mt-1 text-xs text-muted">
                  {(file.detectedFileType ?? "Unrecognized")} · {formatBytes(file.sizeBytes)}
                </p>
              </Link>
              <div className="flex shrink-0 items-center gap-2">
                <Badge tone={file.status}>{file.status}</Badge>
                <button
                  className="rounded-md border border-danger/40 px-3 py-1.5 text-xs font-semibold text-danger transition hover:border-danger hover:bg-danger-strong/20"
                  disabled={isRemoveDisabled}
                  onClick={() => void onRemoveFile(file)}
                  type="button"
                >
                  {deletingFileId === file.id ? "Removing..." : "Remove"}
                </button>
              </div>
            </div>
            <div className="mt-3">
              <button className="text-xs font-semibold uppercase tracking-[0.05em] text-cyan" onClick={() => togglePath(file.id)} type="button">
                {visiblePathIds.has(file.id) ? "Hide path" : "Show path"}
              </button>
              {visiblePathIds.has(file.id) ? <p className="mt-2 break-all technical text-xs text-muted">{file.originalPath}</p> : null}
            </div>
            {file.errorMessage ? <div className="mt-3"><ErrorNotice detail={file.errorMessage} title="File analysis issue" /></div> : null}
          </div>
        ))}
      </div>
    </section>
  );
}

function parentFolder(path: string) {
  const normalized = path.replace(/\\/g, "/");
  const parts = normalized.split("/").filter(Boolean);

  return parts.length > 1 ? parts[parts.length - 2] ?? "Current folder" : "Current folder";
}
