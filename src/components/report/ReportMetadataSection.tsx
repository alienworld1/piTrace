import type { EvidenceFile, FileMetadataGroup, MetadataField } from "../../types/forensics";

interface ReportMetadataSectionProps {
  files: EvidenceFile[];
  metadataByFile: FileMetadataGroup[];
}

export function ReportMetadataSection({ files, metadataByFile }: ReportMetadataSectionProps) {
  const fieldCount = metadataByFile.reduce((count, group) => count + group.fields.length, 0);
  const fileNamesById = new Map(files.map((file) => [file.id, file.fileName]));

  return (
    <div>
      <p className="mb-4 text-sm text-muted">{fieldCount} normalized metadata fields are included in the report package.</p>
      <div className="space-y-3">
        {metadataByFile.map((group) => (
          <article className="rounded-lg border border-line bg-surface p-4" key={group.fileId}>
            <p className="text-sm font-semibold text-ink">{fileNamesById.get(group.fileId) ?? "Unknown evidence file"}</p>
            <div className="mt-3 grid grid-cols-2 gap-2">
              {group.fields.slice(0, 6).map((field) => (
                <MetadataPreviewField field={field} key={field.id} />
              ))}
            </div>
            {group.fields.length > 6 ? <p className="mt-3 text-xs text-muted">+{group.fields.length - 6} more fields in export</p> : null}
          </article>
        ))}
      </div>
    </div>
  );
}

function MetadataPreviewField({ field }: { field: MetadataField }) {
  return (
    <div className="min-w-0 rounded-md border border-line bg-base px-3 py-2">
      <p className="truncate text-xs text-muted">{field.displayLabel ?? field.key}</p>
      <p className="mt-1 truncate text-sm text-ink">{field.value}</p>
    </div>
  );
}
