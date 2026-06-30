# piTrace

piTrace is a local desktop tool for forensic metadata triage. It helps examine files, extract metadata, compute hashes, generate rule-based findings, and export evidence-style reports.

## Features

- Create local cases for groups of evidence files.
- Import supported files by file picker or drag and drop.
- Compute SHA-256 hashes before metadata interpretation.
- Extract metadata locally with bundled ExifTool.
- Group metadata into readable identity, location, timeline, software, and technical sections.
- Generate cautious rule-based findings with severity and confidence levels.
- Preview and export case reports as HTML, JSON, and basic PDF.
- Include or exclude raw metadata and original paths in exported reports.

## Local-First Behavior

piTrace runs as a Tauri desktop app. It stores case records, metadata, findings, and report records in a local SQLite database. Original evidence files stay in place on disk. The app does not upload files.

## Supported Files

Initial supported file categories:

- Images: JPG, JPEG, PNG, TIFF, HEIC
- Documents: PDF, DOCX, PPTX, XLSX
- Audio: MP3, WAV, M4A
- Video: MP4, MOV

Extraction quality depends on the metadata available in each file and ExifTool support for the file format.

## Requirements

- Node.js with pnpm
- Rust stable toolchain
- Tauri v2 system dependencies for your operating system

## Setup

```sh
pnpm install
```

## Development

```sh
pnpm dev
pnpm tauri dev
```

`pnpm dev` starts the Vite frontend. `pnpm tauri dev` starts the desktop app with the Rust backend.

## Build

```sh
pnpm build
pnpm tauri build
```

## Testing

```sh
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features
```

No frontend test runner is configured. For UI changes, run `pnpm build` and smoke test with `pnpm tauri dev`.

## Security Notes

- Treat imported paths and metadata as untrusted input.
- Review raw metadata before sharing reports. It may contain usernames, paths, GPS data, device information, and software history.
- Original paths are excluded from exported reports unless enabled in report options.
- Findings are review indicators. They do not prove authorship, intent, authenticity, or manipulation.

## Limitations

- PDF export is basic and intended for simple review output.
- piTrace does not parse disk images, recover deleted files, inspect memory dumps, or perform malware detection.
- Metadata interpretation should be corroborated with other evidence and case context.
