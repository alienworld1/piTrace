import type { RawMetadataRecord } from "../../types/forensics";
import { Badge } from "../ui/Badge";

interface RawMetadataJsonPanelProps {
  rawMetadata?: RawMetadataRecord | null;
}

export function RawMetadataJsonPanel({ rawMetadata }: RawMetadataJsonPanelProps) {
  return (
    <section className="panel-edge rounded-xl p-5">
      <div className="flex items-start justify-between gap-4">
        <h2 className="font-display text-xl font-semibold text-ink">Raw ExifTool JSON</h2>
        {rawMetadata ? <Badge tone="neutral">{rawMetadata.source}</Badge> : null}
      </div>
      <div className="mt-5 overflow-hidden rounded-lg border border-line bg-base">
        {rawMetadata ? (
          <details>
            <summary className="cursor-pointer px-4 py-4 text-sm font-semibold text-cyan">Show raw metadata</summary>
            <p className="border-t border-line px-4 py-3 text-sm leading-6 text-muted">
              Raw metadata may contain sensitive paths, usernames, GPS data, device details, and software history.
            </p>
            <pre className="max-h-130 overflow-auto border-t border-line p-4 technical text-xs leading-5 text-ink">
              {JSON.stringify(rawMetadata.data, null, 2)}
            </pre>
          </details>
        ) : (
          <p className="px-4 py-5 text-sm text-muted">No raw ExifTool JSON is available yet.</p>
        )}
      </div>
    </section>
  );
}
