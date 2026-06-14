import { useParams } from "react-router";
import { FileIdentityPanel } from "../components/evidence/FileIdentityPanel";
import { MetadataCategoryGrid } from "../components/evidence/MetadataCategoryGrid";
import { MetadataTable } from "../components/evidence/MetadataTable";
import { FindingList } from "../components/findings/FindingList";
import { PanelHeader } from "../components/ui/PanelHeader";
import { getFileById, getFindingsForFile, getMetadataForFile } from "../data/selectors";

export function FileAnalysisPage() {
  const { fileId } = useParams();
  const file = getFileById(fileId);
  const findings = getFindingsForFile(file.id);
  const fields = getMetadataForFile(file.id);

  return (
    <div className="space-y-6">
      <PanelHeader eyebrow="File analysis" title="Metadata review" />
      <FileIdentityPanel file={file} />
      <div className="grid grid-cols-[1fr_420px] gap-6">
        <div className="space-y-6">
          <MetadataCategoryGrid fields={fields} />
          <MetadataTable fields={fields} />
        </div>
        <FindingList caseId={file.caseId} findings={findings} />
      </div>
    </div>
  );
}
