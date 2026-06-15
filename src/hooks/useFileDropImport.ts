import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";

interface UseFileDropImportOptions {
  disabled?: boolean;
  onImport: (filePaths: string[]) => Promise<void>;
}

export function useFileDropImport({ disabled = false, onImport }: UseFileDropImportOptions) {
  const [isDragActive, setIsDragActive] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let isMounted = true;

    async function listenForDrops() {
      unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        if (!isMounted || disabled) {
          return;
        }

        if (event.payload.type === "enter" || event.payload.type === "over") {
          setIsDragActive(true);
          return;
        }

        if (event.payload.type === "leave") {
          setIsDragActive(false);
          return;
        }

        setIsDragActive(false);
        void onImport(event.payload.paths);
      });
    }

    void listenForDrops();

    return () => {
      isMounted = false;
      unlisten?.();
    };
  }, [disabled, onImport]);

  return { isDragActive };
}
