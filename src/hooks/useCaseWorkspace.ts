import { useCallback, useState } from "react";
import { getCase, getCaseFiles, getCaseFindings, getImportConfig, importFiles } from "../services/piTraceApi";
import { useAsyncData } from "./useAsyncData";

export function useCaseWorkspace(caseId: string | undefined) {
  const [importError, setImportError] = useState<string>();
  const [isImporting, setIsImporting] = useState(false);

  const workspace = useAsyncData(async () => {
    if (!caseId) {
      throw new Error("Case id is missing");
    }

    const [caseRecord, files, findings, importConfig] = await Promise.all([
      getCase(caseId),
      getCaseFiles(caseId),
      getCaseFindings(caseId),
      getImportConfig(),
    ]);
    return { caseRecord, files, findings, importConfig };
  }, [caseId]);
  const { reload } = workspace;

  const importPaths = useCallback(
    async (filePaths: string[]) => {
      if (!caseId || filePaths.length === 0) {
        return;
      }

      setIsImporting(true);
      setImportError(undefined);

      try {
        await importFiles(caseId, filePaths);
      } catch (error) {
        setImportError(error instanceof Error ? error.message : String(error));
      } finally {
        await reload();
        setIsImporting(false);
      }
    },
    [caseId, reload],
  );

  return {
    ...workspace,
    importError,
    importPaths,
    isImporting,
  };
}
