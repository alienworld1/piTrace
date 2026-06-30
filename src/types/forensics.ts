export type EvidenceStatus = "pending" | "analyzing" | "complete" | "error";
export type ReportFormat = "html" | "json" | "pdf";
export type Severity = "low" | "medium" | "high";
export type Confidence = "low" | "medium" | "high";
export type FindingCategory =
  | "identity"
  | "location"
  | "timeline"
  | "software"
  | "integrity"
  | "privacy";
export type MetadataCategory =
  | "identity"
  | "location"
  | "timeline"
  | "software"
  | "technical"
  | "integrity"
  | "other";

export interface CaseRecord {
  id: string;
  name: string;
  examinerName?: string;
  notes?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CaseInput {
  name: string;
  examinerName?: string;
  notes?: string;
}

export interface CaseDashboardItem {
  caseRecord: CaseRecord;
  fileCount: number;
  findingCount: number;
  highCount: number;
}

export interface EvidenceFile {
  id: string;
  caseId: string;
  originalPath: string;
  fileName: string;
  extension: string;
  detectedMimeType?: string;
  detectedFileType?: string;
  sizeBytes: number;
  sha256?: string;
  importedAt: string;
  analyzedAt?: string;
  status: EvidenceStatus;
  errorMessage?: string;
}

export interface ImportRejection {
  path: string;
  fileName: string;
  reason: string;
}

export interface ImportBatchResult {
  importedFiles: EvidenceFile[];
  rejectedFiles: ImportRejection[];
}

export interface MetadataField {
  id: string;
  fileId: string;
  group: string;
  key: string;
  displayLabel?: string;
  value: string;
  source: "exiftool" | "tika" | "internal";
  normalizedCategory?: MetadataCategory;
}

export interface RawMetadataRecord {
  fileId: string;
  source: "exiftool" | "tika" | "internal";
  extractedAt: string;
  data: unknown;
}

export interface Finding {
  id: string;
  fileId: string;
  category: FindingCategory;
  title: string;
  description: string;
  severity: Severity;
  confidence: Confidence;
  relatedFieldIds: string[];
  createdAt: string;
}

export interface CaseReport {
  id: string;
  caseId: string;
  generatedAt: string;
  format: ReportFormat;
  includeRawMetadata: boolean;
  outputPath?: string;
}

export interface ReportExportInput {
  caseId: string;
  format: ReportFormat;
  includeRawMetadata: boolean;
  includeOriginalPaths: boolean;
  outputPath: string;
}

export interface ReportExportResult {
  report: CaseReport;
  outputPath: string;
}

export interface FileMetadataGroup {
  fileId: string;
  fields: MetadataField[];
}

export interface ReportSummary {
  evidenceCount: number;
  findingCount: number;
  highCount: number;
  mediumCount: number;
  lowCount: number;
  completeFileCount: number;
  pendingFileCount: number;
  errorFileCount: number;
}

export interface ReportPayload {
  caseRecord: CaseRecord;
  files: EvidenceFile[];
  findings: Finding[];
  metadataByFile: FileMetadataGroup[];
  rawMetadataByFile?: RawMetadataRecord[] | null;
  summary: ReportSummary;
  generatedAt: string;
}

export interface ImportDialogFilter {
  name: string;
  extensions: string[];
}

export interface ImportConfig {
  supportedExtensions: string[];
  dialogFilters: ImportDialogFilter[];
}
