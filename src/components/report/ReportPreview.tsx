import type { ReactNode } from "react";
import type { ReportPayload } from "../../types/forensics";
import { formatDateTime } from "../../utils/format";
import { Badge } from "../ui/Badge";
import { ReportEvidenceTable } from "./ReportEvidenceTable";
import { ReportFindingsSection } from "./ReportFindingsSection";
import { ReportMetadataSection } from "./ReportMetadataSection";
import { ReportSummaryGrid } from "./ReportSummaryGrid";
import { ReportTimelineSection } from "./ReportTimelineSection";

interface ReportPreviewProps {
  payload: ReportPayload;
}

export function ReportPreview({ payload }: ReportPreviewProps) {
  return (
    <section className="panel-edge rounded-xl p-6">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Report preview</p>
          <h2 className="mt-2 font-display text-3xl font-semibold text-ink">{payload.caseRecord.name}</h2>
          <p className="mt-2 text-sm text-muted">Generated preview · {formatDateTime(payload.generatedAt)}</p>
        </div>
        <Badge tone="primary">{payload.rawMetadataByFile ? "Raw appendix on" : "Raw appendix off"}</Badge>
      </div>

      <ReportSummaryGrid summary={payload.summary} />

      <div className="mt-8 space-y-4">
        <PreviewSection title="Evidence files and hashes">
          <ReportEvidenceTable files={payload.files} />
        </PreviewSection>
        <PreviewSection title="Findings">
          <ReportFindingsSection findings={payload.findings} />
        </PreviewSection>
        <PreviewSection title="Timeline">
          <ReportTimelineSection entries={payload.timeline} />
        </PreviewSection>
        <PreviewSection title="Metadata appendix">
          <ReportMetadataSection files={payload.files} metadataByFile={payload.metadataByFile} />
        </PreviewSection>
        {payload.rawMetadataByFile ? (
          <PreviewSection title="Raw metadata appendix">
            <p className="rounded-lg border border-line bg-base px-4 py-4 text-sm leading-6 text-muted">
              {payload.rawMetadataByFile.length} raw metadata records will be included in the exported report. Review before sharing because raw metadata may contain paths, usernames, GPS data, device details, and software history.
            </p>
          </PreviewSection>
        ) : null}
      </div>
    </section>
  );
}

function PreviewSection({ children, title }: { children: ReactNode; title: string }) {
  return (
    <section className="space-y-3">
      <h3 className="font-display text-xl font-semibold text-ink">{title}</h3>
      {children}
    </section>
  );
}
