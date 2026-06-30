import { save } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { exportCaseReport, openExportedReport } from "../../services/piTraceApi";
import type { CaseRecord, ReportFormat } from "../../types/forensics";
import { toErrorMessage } from "../../utils/errors";
import { ActionButton } from "../ui/ActionButton";

interface ReportActionsProps {
  caseRecord: CaseRecord;
  disabled?: boolean;
  includeOriginalPaths: boolean;
  includeRawMetadata: boolean;
  onExported: () => Promise<void>;
  onExportedReportId: (reportId: string | undefined) => void;
  onExportedPath: (outputPath: string | undefined) => void;
  onExportMessage: (message: string | undefined, tone?: "success" | "error") => void;
}

const reportFormats: Array<{ format: ReportFormat; label: string; extension: string }> = [
  { format: "html", label: "Export HTML", extension: "html" },
  { format: "json", label: "Export JSON", extension: "json" },
  { format: "pdf", label: "Export PDF", extension: "pdf" },
];

export function ReportActions({
  caseRecord,
  disabled = false,
  includeOriginalPaths,
  includeRawMetadata,
  onExported,
  onExportedPath,
  onExportedReportId,
  onExportMessage,
}: ReportActionsProps) {
  const [activeFormat, setActiveFormat] = useState<ReportFormat>();

  async function handleExport(format: ReportFormat, extension: string) {
    onExportMessage(undefined);
    onExportedPath(undefined);
    onExportedReportId(undefined);
    setActiveFormat(format);
    try {
      const outputPath = await save({
        defaultPath: `${slugify(caseRecord.name)}-report.${extension}`,
        filters: [{ name: `${format.toUpperCase()} report`, extensions: [extension] }],
      });

      if (!outputPath) {
        return;
      }

      const result = await exportCaseReport({
        caseId: caseRecord.id,
        format,
        includeOriginalPaths,
        includeRawMetadata,
        outputPath,
      });
      await onExported();
      onExportedReportId(result.report.id);
      onExportedPath(result.outputPath);
      onExportMessage("Report exported successfully.", "success");
    } catch (error) {
      onExportMessage(toErrorMessage(error), "error");
    } finally {
      setActiveFormat(undefined);
    }
  }

  return (
    <section className="panel-edge rounded-xl p-5">
      <p className="text-xs font-semibold uppercase tracking-wider text-primary-soft">Export actions</p>
      <div className="mt-4 flex flex-wrap gap-3">
        {reportFormats.map((reportFormat) => (
          <ActionButton
            disabled={disabled || activeFormat !== undefined}
            key={reportFormat.format}
            onClick={() => handleExport(reportFormat.format, reportFormat.extension)}
            variant="technical"
          >
            {activeFormat === reportFormat.format ? "Exporting..." : reportFormat.label}
          </ActionButton>
        ))}
      </div>
    </section>
  );
}

interface ReportExportSuccessActionsProps {
  outputPath: string;
  reportId: string;
  onActionError: (message: string) => void;
}

export function ReportExportSuccessActions({ onActionError, outputPath, reportId }: ReportExportSuccessActionsProps) {
  async function handleOpenReport() {
    try {
      await openExportedReport(reportId);
    } catch (error) {
      onActionError(toErrorMessage(error));
    }
  }

  async function handleShowInFolder() {
    try {
      await revealItemInDir(outputPath);
    } catch (error) {
      onActionError(toErrorMessage(error));
    }
  }

  return (
    <div className="mt-3 flex flex-wrap gap-3">
      <ActionButton onClick={handleOpenReport} variant="technical">
        Open report
      </ActionButton>
      <ActionButton onClick={handleShowInFolder} variant="technical">
        Show in folder
      </ActionButton>
    </div>
  );
}

function slugify(value: string) {
  const slug = value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

  return slug || "pitrace";
}
