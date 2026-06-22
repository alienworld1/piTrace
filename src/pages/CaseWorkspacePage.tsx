import { useNavigate, useParams } from "react-router";
import { CaseSummary } from "../components/cases/CaseSummary";
import { EvidenceList } from "../components/evidence/EvidenceList";
import { ImportDropzone } from "../components/evidence/ImportDropzone";
import { FindingList } from "../components/findings/FindingList";
import { ActionButton } from "../components/ui/ActionButton";
import { EmptyState } from "../components/ui/EmptyState";
import { MetricCard } from "../components/ui/MetricCard";
import { useCaseWorkspace } from "../hooks/useCaseWorkspace";
import { useAsyncAction } from "../hooks/useAsyncAction";
import { deleteCase, deleteFile } from "../services/piTraceApi";
import type { EvidenceFile } from "../types/forensics";

export function CaseWorkspacePage() {
  const { caseId } = useParams();
  const navigate = useNavigate();
  const deletion = useAsyncAction();
  const { data, error, importError, importNotice, importPaths, isImporting, isLoading, reload } = useCaseWorkspace(caseId);
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

  async function handleDeleteCase() {
    if (!caseRecord || !window.confirm(`Remove "${caseRecord.name}" from piTrace? Original evidence files will stay on disk.`)) {
      return;
    }

    const deleted = await deletion.run(`case:${caseRecord.id}`, async () => {
      await deleteCase(caseRecord.id);
    });
    if (deleted) navigate("/");
  }

  async function handleRemoveFile(file: EvidenceFile) {
    if (!window.confirm(`Remove "${file.fileName}" from this case? The original file will stay on disk.`)) {
      return;
    }

    await deletion.run(`file:${file.id}`, async () => {
      await deleteFile(file.id);
      await reload();
    });
  }

  return (
    <div className="space-y-6">
      <CaseSummary
        action={
          <>
            <ActionButton to={`/cases/${caseRecord.id}/edit`} variant="technical">
              Edit case
            </ActionButton>
            <ActionButton disabled={deletion.isRunning} onClick={handleDeleteCase} variant="danger">
              {deletion.activeKey === `case:${caseRecord.id}` ? "Deleting..." : "Delete case"}
            </ActionButton>
          </>
        }
        caseRecord={caseRecord}
      />
      {deletion.error ? <EmptyState description={deletion.error} title="Could not delete local record" /> : null}
      <div className="grid grid-cols-4 gap-4">
        <MetricCard detail="In this case" label="Files" value={String(files.length)} />
        <MetricCard detail="Awaiting analysis" label="Pending" value={String(pendingCount)} />
        <MetricCard detail="Rule-based" label="Findings" value={String(findings.length)} />
        <MetricCard detail="Review first" label="High" value={String(findings.filter((finding) => finding.severity === "high").length)} />
      </div>
      <div className="grid grid-cols-[420px_1fr] gap-6">
        <div className="space-y-6">
          <ImportDropzone config={data?.importConfig} error={importError} isImporting={isImporting} notice={importNotice} onImport={importPaths} />
          <EvidenceList
            deletingFileId={deletion.activeKey?.startsWith("file:") ? deletion.activeKey.slice(5) : undefined}
            files={files}
            isRemoveDisabled={deletion.isRunning}
            onRemoveFile={handleRemoveFile}
          />
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
