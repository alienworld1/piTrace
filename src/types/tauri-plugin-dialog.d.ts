declare module "@tauri-apps/plugin-dialog" {
  interface DialogFilter {
    name: string;
    extensions: string[];
  }

  interface OpenDialogOptions {
    directory?: boolean;
    multiple?: boolean;
    filters?: DialogFilter[];
  }

  export function open(options?: OpenDialogOptions): Promise<string | string[] | null>;
}
