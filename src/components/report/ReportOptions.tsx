interface ReportOptionsProps {
  includeOriginalPaths: boolean;
  includeRawMetadata: boolean;
  onIncludeOriginalPathsChange: (includeOriginalPaths: boolean) => void;
  onIncludeRawMetadataChange: (includeRawMetadata: boolean) => void;
}

export function ReportOptions({
  includeOriginalPaths,
  includeRawMetadata,
  onIncludeOriginalPathsChange,
  onIncludeRawMetadataChange,
}: ReportOptionsProps) {
  return (
    <section className="panel-edge rounded-xl p-5">
      <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Report options</p>
      <div className="mt-4 space-y-3">
        <label className="flex max-w-3xl items-start gap-3 text-sm text-muted">
          <input
            checked={includeRawMetadata}
            className="mt-1 h-4 w-4 accent-cyan"
            onChange={(event) => onIncludeRawMetadataChange(event.currentTarget.checked)}
            type="checkbox"
          />
          <span>
            Include raw metadata appendix. Review before sharing because raw metadata can contain sensitive paths, usernames, GPS data, device details, and software history.
          </span>
        </label>
        <label className="flex max-w-3xl items-start gap-3 text-sm text-muted">
          <input
            checked={includeOriginalPaths}
            className="mt-1 h-4 w-4 accent-cyan"
            onChange={(event) => onIncludeOriginalPathsChange(event.currentTarget.checked)}
            type="checkbox"
          />
          <span>Include original file paths in exported reports. Enable only when the receiving report needs full local path context.</span>
        </label>
      </div>
    </section>
  );
}
