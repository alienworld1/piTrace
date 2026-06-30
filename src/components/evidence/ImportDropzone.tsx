import type { ImportConfig } from "../../types/forensics";
import { ImportPickerButton } from "./ImportPickerButton";
import { useFileDropImport } from "../../hooks/useFileDropImport";
import { ErrorNotice } from "../ui/ErrorNotice";

interface ImportDropzoneProps {
  config: ImportConfig | undefined;
  error?: string;
  notice?: string;
  isImporting: boolean;
  onImport: (filePaths: string[]) => Promise<void>;
}

export function ImportDropzone({ config, error, notice, isImporting, onImport }: ImportDropzoneProps) {
  const { isDragActive } = useFileDropImport({ disabled: isImporting, onImport });
  const supportedText = config?.supportedExtensions.map((extension) => extension.toUpperCase()).join(", ") ?? "Loading supported file types";

  return (
    <section
      className={`rounded-xl border border-dashed p-6 text-center transition ${
        isDragActive ? "border-cyan bg-cyan/10" : "border-primary-soft/50 bg-panel/60"
      }`}
    >
      <p className="technical text-xs uppercase tracking-[0.05em] text-cyan">Read-only import area</p>
      <p className="mt-3 font-display text-lg font-semibold text-ink">{isDragActive ? "Release to import files" : "Drop files for local metadata triage"}</p>
      <p className="mt-2 text-sm leading-6 text-muted">Files stay in place. piTrace records paths and file identity locally.</p>
      <p className="mt-2 technical text-xs text-primary-soft">Supported: {supportedText}</p>
      <div className="mt-5">
        <ImportPickerButton disabled={isImporting || !config} filters={config?.dialogFilters ?? []} onImport={onImport} />
      </div>
      {isImporting ? <p className="mt-4 text-sm text-cyan">Importing files...</p> : null}
      {notice ? <p className="mt-4 text-sm text-cyan">{notice}</p> : null}
      {error ? <div className="mt-4 text-left"><ErrorNotice detail={error} title="Import issue" /></div> : null}
    </section>
  );
}
