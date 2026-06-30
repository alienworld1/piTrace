import { Link } from "react-router";
import type { CaseRecord } from "../../types/forensics";
import { formatDateTime } from "../../utils/format";
import { Badge } from "../ui/Badge";
import { ActionButton } from "../ui/ActionButton";

interface CaseCardProps {
  caseRecord: CaseRecord;
  fileCount: number;
  findingCount: number;
  highCount: number;
  isDeleteDisabled?: boolean;
  isDeleting?: boolean;
  onDelete: (caseRecord: CaseRecord) => void | Promise<void>;
}

export function CaseCard({ caseRecord, fileCount, findingCount, highCount, isDeleteDisabled = false, isDeleting = false, onDelete }: CaseCardProps) {
  return (
    <article className="panel-edge rounded-xl p-5 transition hover:border-cyan/50 hover:bg-panel-high">
      <div className="flex items-start justify-between gap-4">
        <Link className="min-w-0 flex-1" to={`/cases/${caseRecord.id}`}>
          <h2 className="font-display text-xl font-semibold text-ink">{caseRecord.name}</h2>
          <p className="mt-2 text-sm leading-6 text-muted">{caseRecord.notes}</p>
        </Link>
        {highCount > 0 ? <Badge tone="high">{highCount} high</Badge> : <Badge tone="neutral">No high</Badge>}
      </div>
      <div className="mt-6 grid gap-3 text-sm sm:grid-cols-3">
        <div>
          <p className="text-muted">Files</p>
          <p className="mt-1 technical text-ink">{fileCount}</p>
        </div>
        <div>
          <p className="text-muted">Findings</p>
          <p className="mt-1 technical text-ink">{findingCount}</p>
        </div>
        <div>
          <p className="text-muted">Updated</p>
          <p className="mt-1 text-ink">{formatDateTime(caseRecord.updatedAt)}</p>
        </div>
      </div>
      <div className="mt-5 flex gap-3">
        <ActionButton to={`/cases/${caseRecord.id}`} variant="technical">
          Open
        </ActionButton>
        <ActionButton to={`/cases/${caseRecord.id}/edit`} variant="technical">
          Edit
        </ActionButton>
        <ActionButton disabled={isDeleteDisabled} onClick={() => void onDelete(caseRecord)} variant="danger">
          {isDeleting ? "Deleting..." : "Delete"}
        </ActionButton>
      </div>
    </article>
  );
}
