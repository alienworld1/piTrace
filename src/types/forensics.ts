export type EvidenceStatus = "pending" | "hashing" | "analyzing" | "complete" | "error";
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
  md5?: string;
  importedAt: string;
  analyzedAt?: string;
  status: EvidenceStatus;
  errorMessage?: string;
}

export interface MetadataField {
  id: string;
  fileId: string;
  group: string;
  key: string;
  value: string;
  source: "exiftool" | "tika" | "internal";
  normalizedCategory?: MetadataCategory;
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
  format: "html" | "json" | "pdf";
  includeRawMetadata: boolean;
  outputPath?: string;
}
