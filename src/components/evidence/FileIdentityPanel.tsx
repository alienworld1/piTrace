import type { ReactNode } from "react";
import { useState } from "react";
import type { EvidenceFile } from "../../types/forensics";
import { formatBytes, formatDateTime } from "../../utils/format";
import { Badge } from "../ui/Badge";
import { ErrorNotice } from "../ui/ErrorNotice";

interface FileIdentityPanelProps {
  file: EvidenceFile;
  action?: ReactNode;
}

export function FileIdentityPanel({ file, action }: FileIdentityPanelProps) {
  const extensionMismatch = getExtensionMismatch(file);
  const [isPathVisible, setIsPathVisible] = useState(false);
  const location = fileLocation(file.originalPath);

  return (
    <section className="panel-edge rounded-xl p-5">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <p className="text-xs font-semibold uppercase tracking-wider text-primary-soft">File identity</p>
          <h2 className="mt-2 font-display text-3xl font-semibold text-ink">{file.fileName}</h2>
          <p className="mt-2 text-xs text-muted">
            Location: <span className="technical">{location}</span>
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-3">
          <Badge tone={file.status}>{file.status}</Badge>
          {action}
        </div>
      </div>
      <dl className="mt-6 grid gap-4 text-sm sm:grid-cols-2 xl:grid-cols-5">
        <div>
          <dt className="text-muted">Extension</dt>
          <dd className="mt-1 text-ink">{file.extension ? `.${file.extension}` : "None"}</dd>
        </div>
        <div>
          <dt className="text-muted">Type</dt>
          <dd className="mt-1 text-ink">{file.detectedFileType ?? "Unrecognized"}</dd>
        </div>
        <div>
          <dt className="text-muted">MIME</dt>
          <dd className="mt-1 text-ink">{file.detectedMimeType ?? "Unrecognized"}</dd>
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
      <div className="mt-4 rounded-lg border border-line bg-base px-4 py-3">
        <button
          className="text-xs font-semibold uppercase tracking-[0.05em] text-cyan"
          onClick={() => setIsPathVisible((visible) => !visible)}
          type="button"
        >
          {isPathVisible ? "Hide path" : "Show path"}
        </button>
        {isPathVisible ? <p className="mt-3 break-all technical text-xs text-muted">{file.originalPath}</p> : null}
      </div>
      {extensionMismatch ? (
        <div className="mt-4 rounded-lg border border-danger/40 bg-danger-strong/20 p-4 text-sm text-danger">
          Extension mismatch: this file is named <span className="technical">.{extensionMismatch.extension}</span> but its
          content was detected as <span className="technical">{extensionMismatch.detectedType}</span>
          {file.detectedMimeType ? (
            <>
              {" "}
              (<span className="technical">{file.detectedMimeType}</span>)
            </>
          ) : null}
          .
        </div>
      ) : null}
      <div className="mt-5 rounded-lg border border-line bg-base px-4 py-3">
        <p className="text-xs font-semibold uppercase tracking-wider text-muted">SHA-256</p>
        <p className="mt-2 break-all technical text-xs text-ink">{file.sha256 ?? "Not available"}</p>
      </div>
      {file.errorMessage ? <div className="mt-4"><ErrorNotice detail={file.errorMessage} title="File analysis issue" /></div> : null}
    </section>
  );
}

function fileLocation(path: string) {
  const normalized = path.replace(/\\/g, "/");
  const parts = normalized.split("/").filter(Boolean);
  if (parts.length <= 1) {
    return "Current folder";
  }

  return parts[parts.length - 2] ?? "Current folder";
}

function getExtensionMismatch(file: EvidenceFile) {
  if (!file.extension || !file.detectedFileType) {
    return null;
  }

  const extension = file.extension.toLowerCase();
  const detectedType = file.detectedFileType.toLowerCase();
  if (equivalentExtensions(extension).has(detectedType)) {
    return null;
  }

  return {
    extension,
    detectedType: file.detectedFileType.toUpperCase(),
  };
}

function equivalentExtensions(extension: string) {
  const aliases: Record<string, string[]> = {
    jpg: ["jpg", "jpeg"],
    jpeg: ["jpg", "jpeg"],
    tif: ["tif", "tiff"],
    tiff: ["tif", "tiff"],
  };

  return new Set(aliases[extension] ?? [extension]);
}
