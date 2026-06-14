import { CaseCard } from "../components/cases/CaseCard";
import { EmptyState } from "../components/ui/EmptyState";
import { MetricCard } from "../components/ui/MetricCard";
import { PanelHeader } from "../components/ui/PanelHeader";
import { mockCases } from "../data/mockData";
import { getFilesForCase, getFindingsForCase } from "../data/selectors";

export function CaseDashboardPage() {
  const fileCount = mockCases.reduce((total, caseRecord) => total + getFilesForCase(caseRecord.id).length, 0);
  const findingCount = mockCases.reduce((total, caseRecord) => total + getFindingsForCase(caseRecord.id).length, 0);

  return (
    <div className="space-y-6">
      <PanelHeader eyebrow="Case dashboard" title="Local cases" />
      <div className="grid grid-cols-3 gap-4">
        <MetricCard detail="Stored locally" label="Cases" value={String(mockCases.length)} />
        <MetricCard detail="Across mock cases" label="Evidence files" value={String(fileCount)} />
        <MetricCard detail="Rule-based indicators" label="Findings" value={String(findingCount)} />
      </div>
      {mockCases.length === 0 ? (
        <EmptyState description="No cases yet. Create a case to begin local forensic metadata analysis." title="No cases yet" />
      ) : (
        <div className="grid grid-cols-2 gap-4">
          {mockCases.map((caseRecord) => (
            <CaseCard caseRecord={caseRecord} key={caseRecord.id} />
          ))}
        </div>
      )}
    </div>
  );
}
