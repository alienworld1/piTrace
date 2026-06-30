interface ErrorNoticeProps {
  detail?: string;
  title?: string;
}

export function ErrorNotice({ detail, title = "Action could not be completed" }: ErrorNoticeProps) {
  if (!detail) {
    return null;
  }

  const summary = summarizeError(detail);
  const showDetail = summary !== detail;

  return (
    <div className="rounded-lg border border-danger/40 bg-danger-strong/20 p-4 text-sm text-danger">
      <p className="font-semibold">{title}</p>
      <p className="mt-1 leading-6">{summary}</p>
      {showDetail ? (
        <details className="mt-3">
          <summary className="cursor-pointer text-xs font-semibold uppercase tracking-[0.05em]">Technical detail</summary>
          <p className="mt-2 break-words technical text-xs leading-5">{detail}</p>
        </details>
      ) : null}
    </div>
  );
}

function summarizeError(detail: string) {
  const normalized = detail.toLowerCase();

  if (normalized.includes("exiftool")) {
    return "Metadata extraction could not be completed for this file. Review the technical detail or try importing the file again.";
  }

  if (normalized.includes("sqlite")) {
    return "Local case storage could not complete the request. Close other piTrace windows and try again.";
  }

  if (normalized.includes("unsupported file extension")) {
    return "This file type is not enabled for import.";
  }

  if (normalized.includes("file is unavailable") || normalized.includes("no such file")) {
    return "The file could not be read from its current location.";
  }

  if (normalized.includes("symbolic link")) {
    return "This path was rejected for safety.";
  }

  if (normalized.includes("changed while") || normalized.includes("changed during")) {
    return "The file changed during analysis. Re-import it when the file is stable.";
  }

  return detail;
}
