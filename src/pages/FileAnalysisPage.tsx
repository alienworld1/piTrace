import { useNavigate, useParams } from "react-router";
import { FileIdentityPanel } from "../components/evidence/FileIdentityPanel";
import { MetadataCategoryGrid } from "../components/evidence/MetadataCategoryGrid";
import { MetadataTable } from "../components/evidence/MetadataTable";
import { RawMetadataJsonPanel } from "../components/evidence/RawMetadataJsonPanel";
import { FindingList } from "../components/findings/FindingList";
import { ActionButton } from "../components/ui/ActionButton";
import { EmptyState } from "../components/ui/EmptyState";
import { PanelHeader } from "../components/ui/PanelHeader";
import { deleteFile, getFile, getFileFindings, getFileMetadata, getFileRawMetadata } from "../services/piTraceApi";
import { useAsyncData } from "../hooks/useAsyncData";
import { useAsyncAction } from "../hooks/useAsyncAction";

export function FileAnalysisPage() {
  const { fileId } = useParams();
  const navigate = useNavigate();
  const deletion = useAsyncAction();
  const { data, error, isLoading } = useAsyncData(async () => {
    if (!fileId) {
      throw new Error("File id is missing");
    }

    const [file, findings, fields, rawMetadata] = await Promise.all([
      getFile(fileId),
      getFileFindings(fileId),
      getFileMetadata(fileId),
      getFileRawMetadata(fileId),
    ]);
    return { file, findings, fields, rawMetadata };
  }, [fileId]);

  if (isLoading) {
    return <EmptyState description="Loading evidence file." title="Loading file" />;
  }

  if (error || !data) {
    return <EmptyState description={error ?? "Evidence file not found."} title="Could not load file" />;
  }

  const { fields, file, findings, rawMetadata } = data;

  async function handleRemoveFile() {
    if (!window.confirm(`Remove "${file.fileName}" from this case? The original file will stay on disk.`)) {
      return;
    }

    const deleted = await deletion.run(file.id, async () => {
      await deleteFile(file.id);
    });
    if (deleted) navigate(`/cases/${file.caseId}`);
  }

  return (
    <div className="space-y-6">
      <PanelHeader eyebrow="File analysis" title="Metadata review" />
      {deletion.error ? <EmptyState description={deletion.error} title="Could not remove file" /> : null}
      <FileIdentityPanel
        action={
          <ActionButton disabled={deletion.isRunning} onClick={handleRemoveFile} variant="danger">
            {deletion.isRunning ? "Removing..." : "Remove file"}
          </ActionButton>
        }
        file={file}
      />
      <div className="grid gap-6 xl:grid-cols-[minmax(0,1fr)_420px]">
        <div className="min-w-0 space-y-6">
          <MetadataCategoryGrid fields={fields} />
          <RawMetadataJsonPanel rawMetadata={rawMetadata} />
          <MetadataTable fields={fields} />
        </div>
        <FindingList caseId={file.caseId} findings={findings} />
      </div>
    </div>
  );
}
