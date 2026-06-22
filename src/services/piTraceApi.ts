import { invoke } from "@tauri-apps/api/core";
import type {
  CaseInput,
  CaseDashboardItem,
  CaseRecord,
  CaseReport,
  EvidenceFile,
  Finding,
  ImportBatchResult,
  ImportConfig,
  MetadataField,
  RawMetadataRecord,
} from "../types/forensics";

export function listCaseDashboard() {
  return invoke<CaseDashboardItem[]>("list_case_dashboard");
}

export function createCase(input: CaseInput) {
  return invoke<CaseRecord>("create_case", { input });
}

export function updateCase(caseId: string, input: CaseInput) {
  return invoke<CaseRecord>("update_case", { caseId, input });
}

export function deleteCase(caseId: string) {
  return invoke<CaseRecord>("delete_case", { caseId });
}

export function getCase(caseId: string) {
  return invoke<CaseRecord>("get_case", { caseId });
}

export function getCaseFiles(caseId: string) {
  return invoke<EvidenceFile[]>("get_case_files", { caseId });
}

export function getFile(fileId: string) {
  return invoke<EvidenceFile>("get_file", { fileId });
}

export function deleteFile(fileId: string) {
  return invoke<EvidenceFile>("delete_file", { fileId });
}

export function importFiles(caseId: string, filePaths: string[]) {
  return invoke<ImportBatchResult>("import_files", { caseId, filePaths });
}

export function getImportConfig() {
  return invoke<ImportConfig>("get_import_config");
}

export function getCaseFindings(caseId: string) {
  return invoke<Finding[]>("get_case_findings", { caseId });
}

export function getFileFindings(fileId: string) {
  return invoke<Finding[]>("get_file_findings", { fileId });
}

export function getFileMetadata(fileId: string) {
  return invoke<MetadataField[]>("get_file_metadata", { fileId });
}

export function getFileRawMetadata(fileId: string) {
  return invoke<RawMetadataRecord | null>("get_file_raw_metadata", { fileId });
}

export function getFinding(findingId: string) {
  return invoke<Finding>("get_finding", { findingId });
}

export function getCaseReport(caseId: string) {
  return invoke<CaseReport | null>("get_case_report", { caseId });
}
