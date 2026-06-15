import { getCaseFiles, getCaseFindings, listCases } from "../services/piTraceApi";
import type { CaseRecord } from "../types/forensics";
import { useAsyncData } from "./useAsyncData";

export interface CaseDashboardItem {
  caseRecord: CaseRecord;
  fileCount: number;
  findingCount: number;
  highCount: number;
}

export function useCaseDashboard() {
  return useAsyncData(async () => {
    const cases = await listCases();
    const items = await Promise.all(
      cases.map(async (caseRecord) => {
        const [files, findings] = await Promise.all([getCaseFiles(caseRecord.id), getCaseFindings(caseRecord.id)]);
        return {
          caseRecord,
          fileCount: files.length,
          findingCount: findings.length,
          highCount: findings.filter((finding) => finding.severity === "high").length,
        };
      }),
    );

    return {
      cases,
      items,
      fileCount: items.reduce((total, item) => total + item.fileCount, 0),
      findingCount: items.reduce((total, item) => total + item.findingCount, 0),
    };
  }, []);
}
