#!/usr/bin/env bash
# Patch the wasm-pack-generated TypeScript declarations in pkg/hsml.d.ts:
# - rewrite `options: any` JsValue inputs to typed interfaces
# - rewrite the `: any` JsValue return on compileContentWithDiagnostics to CompileResult
# - append the named interface declarations referenced by the rewritten signatures

set -euo pipefail

TARGET="${1:-pkg/hsml.d.ts}"

# `sed -i.bak` is portable across GNU and BSD sed; the .bak file is removed below.
sed -i.bak 's/compileContentWithDiagnostics(source: string, options: any): any/compileContentWithDiagnostics(source: string, options?: CompileOptions): CompileResult/' "$TARGET"
sed -i.bak 's/formatContent(source: string, options: any)/formatContent(source: string, options?: FormatContentOptions)/' "$TARGET"
rm "${TARGET}.bak"

cat >> "$TARGET" <<'EOF'

/** A single point in source code. */
export interface Position {
  /** Line number (1-based). */
  line: number;
  /** Column number (1-based). */
  column: number;
}

/** A span in source code, defined by a start and end position. */
export interface Location {
  /** Start position of the span (inclusive). */
  start: Position;
  /** End position of the span (exclusive). */
  end: Position;
}

/** A diagnostic message (error or warning) from the HSML compiler. */
export interface Diagnostic {
  /** Severity level of this diagnostic. */
  severity: "error" | "warning";
  /** Human-readable description of the issue. */
  message: string;
  /** Error or warning code (e.g. "E001", "W002"). */
  code?: string;
  /** Source location where the issue was found. */
  location?: Location;
  /** File path, if available. */
  filePath?: string;
}

/** Options for compiling HSML source. */
export interface CompileOptions {
  /** Emit pretty-printed HTML with indentation (default: false). */
  pretty?: boolean;
  /** Number of spaces per indentation level (default: 2). */
  indentSize?: number;
}

/** Result of compiling HSML source with diagnostic support. */
export interface CompileResult {
  /** Whether compilation succeeded without errors. */
  success: boolean;
  /** The compiled HTML output, or null if compilation failed. */
  html: string | null;
  /** All diagnostics (errors and warnings) collected during compilation. */
  diagnostics: Diagnostic[];
}

/** Options for formatting HSML source. */
export interface FormatContentOptions {
  /** Number of spaces per indentation level (default: 2). */
  indentSize?: number;
  /** Maximum line width before wrapping attributes (default: 80). */
  printWidth?: number;
}
EOF
