import { useParams } from "react-router";
import { ReportActions } from "../components/report/ReportActions";
import { ReportPreview } from "../components/report/ReportPreview";
import { EmptyState } from "../components/ui/EmptyState";
import { PanelHeader } from "../components/ui/PanelHeader";
import { useAsyncData } from "../hooks/useAsyncData";
import { getCase, getCaseFiles, getCaseFindings, getCaseReport } from "../services/piTraceApi";
import { formatDateTime } from "../utils/format";

export function ReportPreviewPage() {
  const { caseId } = useParams();
  const { data, error, isLoading } = useAsyncData(async () => {
    if (!caseId) {
      throw new Error("Case id is missing");
    }

    const [caseRecord, files, findings, report] = await Promise.all([
      getCase(caseId),
      getCaseFiles(caseId),
      getCaseFindings(caseId),
      getCaseReport(caseId),
    ]);

    return { caseRecord, files, findings, report };
  }, [caseId]);

  if (isLoading) {
    return <EmptyState description="Loading report preview." title="Loading report" />;
  }

  if (error || !data) {
    return <EmptyState description={error ?? "Case not found."} title="Could not load report" />;
  }

  return (
    <div className="space-y-6">
      <PanelHeader eyebrow="Evidence-style report" title="Preview export package" />
      <div className="rounded-xl border border-line bg-panel px-5 py-4 text-sm text-muted">
        {data.report ? (
          <>
            Last preview record: <span className="text-ink">{formatDateTime(data.report.generatedAt)}</span> · Format:{" "}
            <span className="uppercase text-ink">{data.report.format}</span>
          </>
        ) : (
          "No exported report record exists yet."
        )}
      </div>
      <ReportActions />
      <ReportPreview caseRecord={data.caseRecord} files={data.files} findings={data.findings} />
    </div>
  );
}
