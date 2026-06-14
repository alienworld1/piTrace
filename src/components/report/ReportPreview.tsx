import type { CaseRecord, EvidenceFile, Finding } from "../../types/forensics";
import { formatDateTime } from "../../utils/format";
import { Badge } from "../ui/Badge";

interface ReportPreviewProps {
  caseRecord: CaseRecord;
  files: EvidenceFile[];
  findings: Finding[];
}

export function ReportPreview({ caseRecord, files, findings }: ReportPreviewProps) {
  return (
    <section className="panel-edge rounded-xl p-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Report preview</p>
          <h2 className="mt-2 font-display text-3xl font-semibold text-ink">{caseRecord.name}</h2>
          <p className="mt-2 text-sm text-muted">Generated preview · {formatDateTime(new Date().toISOString())}</p>
        </div>
        <Badge tone="primary">Raw appendix on</Badge>
      </div>

      <div className="mt-8 grid grid-cols-3 gap-4">
        <div className="rounded-lg border border-line bg-base p-4">
          <p className="text-muted">Evidence items</p>
          <p className="mt-2 technical text-2xl text-ink">{files.length}</p>
        </div>
        <div className="rounded-lg border border-line bg-base p-4">
          <p className="text-muted">Findings</p>
          <p className="mt-2 technical text-2xl text-ink">{findings.length}</p>
        </div>
        <div className="rounded-lg border border-line bg-base p-4">
          <p className="text-muted">High severity</p>
          <p className="mt-2 technical text-2xl text-danger">{findings.filter((finding) => finding.severity === "high").length}</p>
        </div>
      </div>

      <div className="mt-8 space-y-4">
        {files.map((file) => (
          <article className="rounded-lg border border-line bg-surface p-4" key={file.id}>
            <div className="flex items-center justify-between">
              <h3 className="font-display text-lg font-semibold text-ink">{file.fileName}</h3>
              <Badge tone={file.status}>{file.status}</Badge>
            </div>
            <p className="mt-2 text-sm text-muted">{file.detectedFileType ?? "Analysis pending"}</p>
          </article>
        ))}
      </div>
    </section>
  );
}
