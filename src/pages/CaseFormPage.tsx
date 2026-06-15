import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { useNavigate, useParams } from "react-router";
import { ActionButton } from "../components/ui/ActionButton";
import { EmptyState } from "../components/ui/EmptyState";
import { PanelHeader } from "../components/ui/PanelHeader";
import { createCase, getCase, updateCase } from "../services/piTraceApi";

interface CaseFormPageProps {
  mode: "create" | "edit";
}

export function CaseFormPage({ mode }: CaseFormPageProps) {
  const { caseId } = useParams();
  const navigate = useNavigate();
  const [name, setName] = useState("");
  const [examinerName, setExaminerName] = useState("");
  const [notes, setNotes] = useState("");
  const [error, setError] = useState<string>();
  const [isLoading, setIsLoading] = useState(mode === "edit");
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    if (mode !== "edit" || !caseId) {
      return;
    }

    const activeCaseId = caseId;

    async function loadCase() {
      setIsLoading(true);
      setError(undefined);

      try {
        const caseRecord = await getCase(activeCaseId);
        setName(caseRecord.name);
        setExaminerName(caseRecord.examinerName ?? "");
        setNotes(caseRecord.notes ?? "");
      } catch (error) {
        setError(error instanceof Error ? error.message : String(error));
      } finally {
        setIsLoading(false);
      }
    }

    void loadCase();
  }, [caseId, mode]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSaving(true);
    setError(undefined);

    try {
      const input = { name, examinerName, notes };
      const savedCase = mode === "edit" && caseId ? await updateCase(caseId, input) : await createCase(input);
      navigate(`/cases/${savedCase.id}`);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="space-y-6">
      <PanelHeader eyebrow={mode === "create" ? "New case" : "Edit case"} title={mode === "create" ? "Create case" : "Edit case"} />
      {isLoading ? <EmptyState description="Loading case details." title="Loading case" /> : null}
      {error ? <EmptyState description={error} title="Case form error" /> : null}
      {!isLoading ? (
        <form className="panel-edge max-w-3xl rounded-xl p-6" onSubmit={handleSubmit}>
          <label className="block">
            <span className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Case name required</span>
            <input
              className="mt-2 w-full rounded-lg border border-line bg-base px-4 py-3 text-ink outline-none transition focus:border-cyan focus:ring-2 focus:ring-cyan/10"
              onChange={(event) => setName(event.target.value)}
              placeholder="Enter case name"
              required
              value={name}
            />
          </label>
          <label className="mt-5 block">
            <span className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Examiner name</span>
            <input
              className="mt-2 w-full rounded-lg border border-line bg-base px-4 py-3 text-ink outline-none transition focus:border-cyan focus:ring-2 focus:ring-cyan/10"
              onChange={(event) => setExaminerName(event.target.value)}
              placeholder="Optional"
              value={examinerName}
            />
          </label>
          <label className="mt-5 block">
            <span className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Notes</span>
            <textarea
              className="mt-2 min-h-36 w-full resize-none rounded-lg border border-line bg-base px-4 py-3 text-ink outline-none transition focus:border-cyan focus:ring-2 focus:ring-cyan/10"
              onChange={(event) => setNotes(event.target.value)}
              placeholder="Optional case notes"
              value={notes}
            />
          </label>
          <div className="mt-6 flex gap-3">
            <ActionButton disabled={isSaving} type="submit" variant="primary">
              {mode === "create" ? "Create case" : "Save changes"}
            </ActionButton>
            <ActionButton to={caseId ? `/cases/${caseId}` : "/"}>Cancel</ActionButton>
          </div>
        </form>
      ) : null}
    </div>
  );
}
