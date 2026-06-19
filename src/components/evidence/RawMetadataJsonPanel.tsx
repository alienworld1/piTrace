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
          <pre className="max-h-130 overflow-auto p-4 technical text-xs leading-5 text-ink">
            {JSON.stringify(rawMetadata.data, null, 2)}
          </pre>
        ) : (
          <p className="px-4 py-5 text-sm text-muted">No raw ExifTool JSON is available yet.</p>
        )}
      </div>
    </section>
  );
}
