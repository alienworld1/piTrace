import { Link } from "react-router";
import type { CaseRecord } from "../../types/forensics";
import { formatDateTime } from "../../utils/format";
import { Badge } from "../ui/Badge";

interface CaseCardProps {
  caseRecord: CaseRecord;
  fileCount: number;
  findingCount: number;
  highCount: number;
}

export function CaseCard({ caseRecord, fileCount, findingCount, highCount }: CaseCardProps) {
  return (
    <Link className="panel-edge block rounded-xl p-5 transition hover:border-cyan/50 hover:bg-panel-high" to={`/cases/${caseRecord.id}`}>
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="font-display text-xl font-semibold text-ink">{caseRecord.name}</h2>
          <p className="mt-2 text-sm leading-6 text-muted">{caseRecord.notes}</p>
        </div>
        {highCount > 0 ? <Badge tone="high">{highCount} high</Badge> : <Badge tone="neutral">No high</Badge>}
      </div>
      <div className="mt-6 grid grid-cols-3 gap-3 text-sm">
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
    </Link>
  );
}
