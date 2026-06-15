import { open } from "@tauri-apps/plugin-dialog";
import type { ImportDialogFilter } from "../../types/forensics";
import { ActionButton } from "../ui/ActionButton";

interface ImportPickerButtonProps {
  disabled?: boolean;
  filters: ImportDialogFilter[];
  onImport: (filePaths: string[]) => Promise<void>;
}

export function ImportPickerButton({ disabled = false, filters, onImport }: ImportPickerButtonProps) {
  async function handleSelectFiles() {
    const selected = await open({
      directory: false,
      multiple: true,
      filters,
    });

    if (!selected) {
      return;
    }

    await onImport(Array.isArray(selected) ? selected : [selected]);
  }

  return (
    <ActionButton disabled={disabled} onClick={handleSelectFiles} variant="technical">
      Select files
    </ActionButton>
  );
}
