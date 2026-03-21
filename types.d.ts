/**
 * Source location in the HSML file.
 */
export interface Location {
  /** Line number (1-based). */
  line: number
  /** Column number (1-based). */
  column: number
}

/**
 * A diagnostic message (error or warning).
 */
export interface Diagnostic {
  /** Severity level. */
  severity: 'error' | 'warning'
  /** Human-readable description. */
  message: string
  /** Machine-readable error code (e.g., "E001", "W001"). */
  code?: string
  /** Source location where the diagnostic occurred. */
  location?: Location
  /** File path, if available. */
  filePath?: string
}

/**
 * Result of compiling HSML source with diagnostics.
 */
export interface CompileResult {
  /** Whether compilation succeeded. */
  success: boolean
  /** The compiled HTML output, or null on error. */
  html: string | null
  /** Diagnostics (errors on failure, warnings on success). */
  diagnostics: Diagnostic[]
}

/**
 * Compile HSML source to HTML.
 * @throws {Error} on parse or compile errors.
 */
export function compileContent(source: string): string

/**
 * Compile HSML source and return structured result with diagnostics.
 * Never throws — errors are returned in the diagnostics array.
 */
export function compileContentWithDiagnostics(source: string): CompileResult
