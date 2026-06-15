import { useParams } from "react-router";
import { FindingDetail } from "../components/findings/FindingDetail";
import { ActionButton } from "../components/ui/ActionButton";
import { EmptyState } from "../components/ui/EmptyState";
import { PanelHeader } from "../components/ui/PanelHeader";
import { useAsyncData } from "../hooks/useAsyncData";
import { getFile, getFileMetadata, getFinding } from "../services/piTraceApi";

export function FindingDetailPage() {
  const { findingId } = useParams();
  const { data, error, isLoading } = useAsyncData(async () => {
    if (!findingId) {
      throw new Error("Finding id is missing");
    }

    const finding = await getFinding(findingId);
    const [file, metadataFields] = await Promise.all([getFile(finding.fileId), getFileMetadata(finding.fileId)]);
    const fields = metadataFields.filter((field) => finding.relatedFieldIds.includes(field.id));
    return { fields, file, finding };
  }, [findingId]);

  if (isLoading) {
    return <EmptyState description="Loading finding details." title="Loading finding" />;
  }

  if (error || !data) {
    return <EmptyState description={error ?? "Finding not found."} title="Could not load finding" />;
  }

  return (
    <div className="space-y-6">
      <PanelHeader
        action={<ActionButton to={`/cases/${data.file.caseId}/files/${data.file.id}`}>Back to file</ActionButton>}
        eyebrow="Indicator review"
        title={data.file.fileName}
      />
      <FindingDetail finding={data.finding} relatedFields={data.fields} />
    </div>
  );
}
