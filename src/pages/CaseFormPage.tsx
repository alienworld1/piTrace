import { useParams } from "react-router";
import { ActionButton } from "../components/ui/ActionButton";
import { PanelHeader } from "../components/ui/PanelHeader";
import { getCaseById } from "../data/selectors";

interface CaseFormPageProps {
  mode: "create" | "edit";
}

export function CaseFormPage({ mode }: CaseFormPageProps) {
  const { caseId } = useParams();
  const caseRecord = mode === "edit" ? getCaseById(caseId) : undefined;

  return (
    <div className="space-y-6">
      <PanelHeader eyebrow={mode === "create" ? "New case" : "Edit case"} title={mode === "create" ? "Create case shell" : "Edit case shell"} />
      <form className="panel-edge max-w-3xl rounded-xl p-6">
        <label className="block">
          <span className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Case name required</span>
          <input
            className="mt-2 w-full rounded-lg border border-line bg-base px-4 py-3 text-ink outline-none transition focus:border-cyan focus:ring-2 focus:ring-cyan/10"
            defaultValue={caseRecord?.name ?? ""}
            placeholder="Enter case name"
          />
        </label>
        <label className="mt-5 block">
          <span className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Examiner name</span>
          <input
            className="mt-2 w-full rounded-lg border border-line bg-base px-4 py-3 text-ink outline-none transition focus:border-cyan focus:ring-2 focus:ring-cyan/10"
            defaultValue={caseRecord?.examinerName ?? ""}
            placeholder="Optional"
          />
        </label>
        <label className="mt-5 block">
          <span className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Notes</span>
          <textarea
            className="mt-2 min-h-36 w-full resize-none rounded-lg border border-line bg-base px-4 py-3 text-ink outline-none transition focus:border-cyan focus:ring-2 focus:ring-cyan/10"
            defaultValue={caseRecord?.notes ?? ""}
            placeholder="Optional case notes"
          />
        </label>
        <div className="mt-6 flex gap-3">
          <ActionButton disabled variant="primary">
            {mode === "create" ? "Create case" : "Save changes"}
          </ActionButton>
          <ActionButton to={caseRecord ? `/cases/${caseRecord.id}` : "/"}>Cancel</ActionButton>
        </div>
      </form>
    </div>
  );
}
