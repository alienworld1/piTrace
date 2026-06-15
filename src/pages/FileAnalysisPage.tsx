import { useParams } from "react-router";
import { FileIdentityPanel } from "../components/evidence/FileIdentityPanel";
import { MetadataCategoryGrid } from "../components/evidence/MetadataCategoryGrid";
import { MetadataTable } from "../components/evidence/MetadataTable";
import { FindingList } from "../components/findings/FindingList";
import { EmptyState } from "../components/ui/EmptyState";
import { PanelHeader } from "../components/ui/PanelHeader";
import { getFile, getFileFindings, getFileMetadata } from "../services/piTraceApi";
import { useAsyncData } from "../hooks/useAsyncData";

export function FileAnalysisPage() {
  const { fileId } = useParams();
  const { data, error, isLoading } = useAsyncData(async () => {
    if (!fileId) {
      throw new Error("File id is missing");
    }

    const [file, findings, fields] = await Promise.all([getFile(fileId), getFileFindings(fileId), getFileMetadata(fileId)]);
    return { file, findings, fields };
  }, [fileId]);

  if (isLoading) {
    return <EmptyState description="Loading evidence file." title="Loading file" />;
  }

  if (error || !data) {
    return <EmptyState description={error ?? "Evidence file not found."} title="Could not load file" />;
  }

  return (
    <div className="space-y-6">
      <PanelHeader eyebrow="File analysis" title="Metadata review" />
      <FileIdentityPanel file={data.file} />
      <div className="grid grid-cols-[1fr_420px] gap-6">
        <div className="space-y-6">
          <MetadataCategoryGrid fields={data.fields} />
          <MetadataTable fields={data.fields} />
        </div>
        <FindingList caseId={data.file.caseId} findings={data.findings} />
      </div>
    </div>
  );
}
