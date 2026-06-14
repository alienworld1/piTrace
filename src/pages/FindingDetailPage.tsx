import { useParams } from "react-router";
import { FindingDetail } from "../components/findings/FindingDetail";
import { ActionButton } from "../components/ui/ActionButton";
import { PanelHeader } from "../components/ui/PanelHeader";
import { getFileById, getFindingById, getMetadataForFile } from "../data/selectors";

export function FindingDetailPage() {
  const { findingId } = useParams();
  const finding = getFindingById(findingId);
  const file = getFileById(finding.fileId);
  const fields = getMetadataForFile(finding.fileId).filter((field) => finding.relatedFieldIds.includes(field.id));

  return (
    <div className="space-y-6">
      <PanelHeader
        action={<ActionButton to={`/cases/${file.caseId}/files/${file.id}`}>Back to file</ActionButton>}
        eyebrow="Indicator review"
        title={file.fileName}
      />
      <FindingDetail finding={finding} relatedFields={fields} />
    </div>
  );
}
