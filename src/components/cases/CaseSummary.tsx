import type { ReactNode } from "react";
import type { CaseRecord } from "../../types/forensics";
import { formatDateTime } from "../../utils/format";

interface CaseSummaryProps {
  caseRecord: CaseRecord;
  action?: ReactNode;
}

export function CaseSummary({ caseRecord, action }: CaseSummaryProps) {
  return (
    <section className="panel-edge rounded-xl p-5">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0">
          <p className="text-xs font-semibold uppercase tracking-[0.05em] text-primary-soft">Case summary</p>
          <h2 className="mt-2 font-display text-3xl font-semibold text-ink">{caseRecord.name}</h2>
          <p className="mt-3 max-w-3xl text-sm leading-6 text-muted">{caseRecord.notes}</p>
        </div>
        {action ? <div className="flex shrink-0 gap-3">{action}</div> : null}
      </div>
      <dl className="mt-6 grid gap-4 text-sm md:grid-cols-3">
        <div>
          <dt className="text-muted">Examiner</dt>
          <dd className="mt-1 text-ink">{caseRecord.examinerName ?? "Not assigned"}</dd>
        </div>
        <div>
          <dt className="text-muted">Created</dt>
          <dd className="mt-1 text-ink">{formatDateTime(caseRecord.createdAt)}</dd>
        </div>
        <div>
          <dt className="text-muted">Updated</dt>
          <dd className="mt-1 text-ink">{formatDateTime(caseRecord.updatedAt)}</dd>
        </div>
      </dl>
    </section>
  );
}
