import { useParams } from "react-router";
import { CaseSummary } from "../components/cases/CaseSummary";
import { EvidenceList } from "../components/evidence/EvidenceList";
import { ImportDropzone } from "../components/evidence/ImportDropzone";
import { FindingList } from "../components/findings/FindingList";
import { ActionButton } from "../components/ui/ActionButton";
import { EmptyState } from "../components/ui/EmptyState";
import { MetricCard } from "../components/ui/MetricCard";
import { useCaseWorkspace } from "../hooks/useCaseWorkspace";

export function CaseWorkspacePage() {
  const { caseId } = useParams();
  const { data, error, importError, importPaths, isImporting, isLoading } = useCaseWorkspace(caseId);
  const caseRecord = data?.caseRecord;
  const files = data?.files ?? [];
  const findings = data?.findings ?? [];
  const pendingCount = files.filter((file) => file.status === "pending").length;

  if (isLoading) {
    return <EmptyState description="Loading case workspace." title="Loading case" />;
  }

  if (error || !caseRecord) {
    return <EmptyState description={error ?? "Case not found."} title="Could not load case" />;
  }

  return (
    <div className="space-y-6">
      <CaseSummary caseRecord={caseRecord} />
      <div className="grid grid-cols-4 gap-4">
        <MetricCard detail="In this case" label="Files" value={String(files.length)} />
        <MetricCard detail="Awaiting analysis" label="Pending" value={String(pendingCount)} />
        <MetricCard detail="Rule-based" label="Findings" value={String(findings.length)} />
        <MetricCard detail="Review first" label="High" value={String(findings.filter((finding) => finding.severity === "high").length)} />
      </div>
      <div className="grid grid-cols-[420px_1fr] gap-6">
        <div className="space-y-6">
          <ImportDropzone config={data?.importConfig} error={importError} isImporting={isImporting} onImport={importPaths} />
          <EvidenceList files={files} />
        </div>
        <div className="space-y-6">
          <section className="panel-edge rounded-xl p-5">
            <div className="flex items-center justify-between gap-4">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Case actions</p>
                <h2 className="mt-2 font-display text-xl font-semibold text-ink">Analysis workflow</h2>
              </div>
              <div className="flex gap-3">
                <ActionButton disabled variant="technical">
                  Analyze pending
                </ActionButton>
                <ActionButton to={`/cases/${caseRecord.id}/report`} variant="technical">
                  Preview report
                </ActionButton>
              </div>
            </div>
          </section>
          <FindingList caseId={caseRecord.id} findings={findings} />
        </div>
      </div>
    </div>
  );
}
