import { CaseCard } from "../components/cases/CaseCard";
import { EmptyState } from "../components/ui/EmptyState";
import { MetricCard } from "../components/ui/MetricCard";
import { PanelHeader } from "../components/ui/PanelHeader";
import { useCaseDashboard } from "../hooks/useCaseDashboard";
import { deleteCase } from "../services/piTraceApi";
import type { CaseRecord } from "../types/forensics";

export function CaseDashboardPage() {
  const { data, error, isLoading, reload } = useCaseDashboard();
  const cases = data?.cases ?? [];
  const items = data?.items ?? [];

  async function handleDeleteCase(caseRecord: CaseRecord) {
    if (!window.confirm(`Remove "${caseRecord.name}" from piTrace? Original evidence files will stay on disk.`)) {
      return;
    }

    await deleteCase(caseRecord.id);
    await reload();
  }

  return (
    <div className="space-y-6">
      <PanelHeader eyebrow="Case dashboard" title="Local cases" />
      <div className="grid grid-cols-3 gap-4">
        <MetricCard detail="Stored locally" label="Cases" value={String(cases.length)} />
        <MetricCard detail="Across local cases" label="Evidence files" value={String(data?.fileCount ?? 0)} />
        <MetricCard detail="Rule-based indicators" label="Findings" value={String(data?.findingCount ?? 0)} />
      </div>
      {isLoading ? <EmptyState description="Loading local case records." title="Loading cases" /> : null}
      {error ? <EmptyState description={error} title="Could not load cases" /> : null}
      {!isLoading && !error && cases.length === 0 ? (
        <EmptyState description="No cases yet. Create a case to begin local forensic metadata analysis." title="No cases yet" />
      ) : null}
      {!isLoading && !error && cases.length > 0 ? (
        <div className="grid grid-cols-2 gap-4">
          {items.map((item) => (
            <CaseCard
              caseRecord={item.caseRecord}
              fileCount={item.fileCount}
              findingCount={item.findingCount}
              highCount={item.highCount}
              key={item.caseRecord.id}
              onDelete={handleDeleteCase}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}
