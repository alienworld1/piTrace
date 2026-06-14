import { useParams } from "react-router";
import { ReportActions } from "../components/report/ReportActions";
import { ReportPreview } from "../components/report/ReportPreview";
import { PanelHeader } from "../components/ui/PanelHeader";
import { getCaseById, getFilesForCase, getFindingsForCase, getReportForCase } from "../data/selectors";
import { formatDateTime } from "../utils/format";

export function ReportPreviewPage() {
  const { caseId } = useParams();
  const caseRecord = getCaseById(caseId);
  const files = getFilesForCase(caseRecord.id);
  const findings = getFindingsForCase(caseRecord.id);
  const report = getReportForCase(caseRecord.id);

  return (
    <div className="space-y-6">
      <PanelHeader eyebrow="Evidence-style report" title="Preview export package" />
      <div className="rounded-xl border border-line bg-panel px-5 py-4 text-sm text-muted">
        Last preview record: <span className="text-ink">{formatDateTime(report.generatedAt)}</span> · Format:{" "}
        <span className="uppercase text-ink">{report.format}</span>
      </div>
      <ReportActions />
      <ReportPreview caseRecord={caseRecord} files={files} findings={findings} />
    </div>
  );
}
