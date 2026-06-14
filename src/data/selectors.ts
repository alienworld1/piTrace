import { mockCases, mockEvidenceFiles, mockFindings, mockMetadataFields, mockReports } from "./mockData";

export function getCaseById(caseId: string | undefined) {
  return mockCases.find((caseRecord) => caseRecord.id === caseId) ?? mockCases[0];
}

export function getFilesForCase(caseId: string) {
  return mockEvidenceFiles.filter((file) => file.caseId === caseId);
}

export function getFileById(fileId: string | undefined) {
  return mockEvidenceFiles.find((file) => file.id === fileId) ?? mockEvidenceFiles[0];
}

export function getFindingsForFile(fileId: string) {
  return mockFindings.filter((finding) => finding.fileId === fileId);
}

export function getFindingsForCase(caseId: string) {
  const caseFileIds = new Set(getFilesForCase(caseId).map((file) => file.id));
  return mockFindings.filter((finding) => caseFileIds.has(finding.fileId));
}

export function getFindingById(findingId: string | undefined) {
  return mockFindings.find((finding) => finding.id === findingId) ?? mockFindings[0];
}

export function getMetadataForFile(fileId: string) {
  return mockMetadataFields.filter((field) => field.fileId === fileId);
}

export function getReportForCase(caseId: string) {
  return mockReports.find((report) => report.caseId === caseId) ?? mockReports[0];
}
