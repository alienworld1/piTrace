import { useState } from "react";
import { useParams } from "react-router";
import { ReportActions, ReportExportSuccessActions } from "../components/report/ReportActions";
import { ReportOptions } from "../components/report/ReportOptions";
import { ReportPreview } from "../components/report/ReportPreview";
import { EmptyState } from "../components/ui/EmptyState";
import { PanelHeader } from "../components/ui/PanelHeader";
import { useAsyncData } from "../hooks/useAsyncData";
import { getCaseReport, getCaseReportPayload } from "../services/piTraceApi";
import { formatDateTime } from "../utils/format";

export function ReportPreviewPage() {
  const { caseId } = useParams();
  const [includeRawMetadata, setIncludeRawMetadata] = useState(true);
  const [exportMessage, setExportMessage] = useState<string>();
  const [exportTone, setExportTone] = useState<"success" | "error">("success");
  const [lastExportPath, setLastExportPath] = useState<string>();
  const [lastExportReportId, setLastExportReportId] = useState<string>();
  const { data, error, isLoading, reload } = useAsyncData(async () => {
    if (!caseId) {
      throw new Error("Case id is missing");
    }

    const [payload, report] = await Promise.all([
      getCaseReportPayload(caseId, includeRawMetadata),
      getCaseReport(caseId),
    ]);

    return { payload, report };
  }, [caseId, includeRawMetadata]);

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
      <ReportOptions includeRawMetadata={includeRawMetadata} onIncludeRawMetadataChange={setIncludeRawMetadata} />
      <ReportActions
        caseRecord={data.payload.caseRecord}
        includeRawMetadata={includeRawMetadata}
        onExported={reload}
        onExportedPath={setLastExportPath}
        onExportedReportId={setLastExportReportId}
        onExportMessage={(message, tone = "success") => {
          setExportMessage(message);
          setExportTone(tone);
        }}
      />
      {exportMessage ? (
        <section
          className={`rounded-xl border px-5 py-4 text-sm ${
            exportTone === "error" ? "border-danger/60 bg-danger-strong/10 text-danger" : "border-success/50 bg-success/10 text-success"
          }`}
        >
          {exportMessage}
          {lastExportPath && lastExportReportId && exportTone === "success" ? (
            <ReportExportSuccessActions
              onActionError={(message) => {
                setExportMessage(message);
                setExportTone("error");
              }}
              outputPath={lastExportPath}
              reportId={lastExportReportId}
            />
          ) : null}
        </section>
      ) : null}
      <ReportPreview payload={data.payload} />
    </div>
  );
}
