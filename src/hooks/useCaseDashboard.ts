import { listCaseDashboard } from "../services/piTraceApi";
import { useAsyncData } from "./useAsyncData";

export function useCaseDashboard() {
  return useAsyncData(async () => {
    const items = await listCaseDashboard();

    return {
      cases: items.map((item) => item.caseRecord),
      items,
      fileCount: items.reduce((total, item) => total + item.fileCount, 0),
      findingCount: items.reduce((total, item) => total + item.findingCount, 0),
    };
  }, []);
}
