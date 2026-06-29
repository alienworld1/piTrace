interface ReportOptionsProps {
  includeRawMetadata: boolean;
  onIncludeRawMetadataChange: (includeRawMetadata: boolean) => void;
}

export function ReportOptions({ includeRawMetadata, onIncludeRawMetadataChange }: ReportOptionsProps) {
  return (
    <section className="panel-edge rounded-xl p-5">
      <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Report options</p>
      <label className="mt-4 flex max-w-3xl items-start gap-3 text-sm text-muted">
        <input
          checked={includeRawMetadata}
          className="mt-1 h-4 w-4 accent-cyan"
          onChange={(event) => onIncludeRawMetadataChange(event.currentTarget.checked)}
          type="checkbox"
        />
        <span>
          Include raw metadata appendix. This can contain sensitive paths, usernames, GPS data, and software history.
        </span>
      </label>
    </section>
  );
}
