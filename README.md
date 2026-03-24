[![Crate](https://img.shields.io/crates/v/hsml.svg)](https://crates.io/crates/hsml)
[![Downloads](https://img.shields.io/crates/d/hsml.svg)](https://crates.io/crates/hsml)
[![NPM](https://img.shields.io/npm/v/hsml.svg)](https://www.npmjs.com/package/hsml)
[![Downloads](https://img.shields.io/npm/dt/hsml.svg)](https://www.npmjs.com/package/hsml)
[![CI](https://github.com/hsml-lab/hsml/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/hsml-lab/hsml/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/hsml-lab/hsml/branch/main/graph/badge.svg)](https://codecov.io/gh/hsml-lab/hsml)
[![License: MIT](https://img.shields.io/github/license/hsml-lab/hsml.svg)](https://github.com/hsml-lab/hsml/blob/main/LICENSE)
[![Donate: PayPal](https://img.shields.io/badge/Donate-PayPal-blue.svg)](https://www.paypal.com/donate?hosted_button_id=L7GY729FBKTZY)

# HSML - Hyper Short Markup Language

A pug-inspired HTML preprocessor, written in Rust. Less typing, more shipping.

> Still young and growing! Some features are cooking. Check out the [roadmap](#roadmap) below.

## What is it?

HSML compiles a short, indentation-based syntax into HTML — think [Pug](https://pugjs.org), but leaner:

- **TailwindCSS-friendly** — arbitrary values like `.bg-[#1da1f2]` and `lg:[&:nth-child(3)]:hover:underline` just work
- **No template engine** — HSML is to HTML what TypeScript is to JavaScript: a compile-time transformation, no runtime
- **Rust-powered** — fast native CLI + WASM package for browser and bundler use
- **Helpful diagnostics** — descriptive errors and warnings with source context, like a good compiler should

## Quick taste

```hsml
doctype html
html
  head
    title My Page
  body
    h1.text-xl.font-bold Hello World
    .card
      img.rounded-full(src="/avatar.jpg" alt="Me")
      p.text-gray-500 Nice to meet you!
```

Compiles to:

```html
<!DOCTYPE html>
<html>
  <head>
    <title>My Page</title>
  </head>
  <body>
    <h1 class="text-xl font-bold">Hello World</h1>
    <div class="card">
      <img class="rounded-full" src="/avatar.jpg" alt="Me" />
      <p class="text-gray-500">Nice to meet you!</p>
    </div>
  </body>
</html>
```

## Installation

### CLI (via Cargo)

```sh
cargo install hsml
```

### WASM (via npm)

```sh
npm install -D hsml
# or
pnpm add -D hsml
# or
bun add -D hsml
```

## Usage

### CLI

```sh
# Compile a single file
hsml compile index.hsml

# Compile to a specific output file
hsml compile index.hsml -o output.html

# Compile all .hsml files in a directory
hsml compile src/

# Check files for errors and warnings without compiling
hsml check src/

# Get diagnostics as JSON (for CI integration)
hsml compile index.hsml --report-format json
hsml check src/ --report-format json
```

### Ignore patterns

When compiling or checking a directory, HSML automatically skips:

- **Hidden files and directories** (e.g. `.git/`, `.cache/`)
- **Built-in ignores**: `node_modules/`, `target/`, `dist/`, `build/`, `.hg/`, `.svn/`
- **`.gitignore` patterns** (works even outside git repositories)
- **`.hsmlignore` patterns** (same format as `.gitignore`)

You can also pass ignore patterns via CLI:

```sh
hsml compile src/ --ignore-pattern "vendor/"
hsml compile src/ --ignore-pattern "tmp/" --ignore-pattern "generated/"
```

To re-include a built-in ignored directory, add a negation to `.hsmlignore`:

```gitignore
# .hsmlignore
!build/
```

### WASM / JavaScript

```js
import { compileContent, compileContentWithDiagnostics } from "hsml";

// Simple compilation
const html = compileContent("h1.title Hello World\n");
// => '<h1 class="title">Hello World</h1>'

// With diagnostics (errors + warnings)
const result = compileContentWithDiagnostics("h1.foo.foo Hello\n");
// => { success: true, html: '...', diagnostics: [{ severity: 'warning', code: 'W002', ... }] }
```

## HSML Syntax

### Tags

```hsml
h1 Hello World
div
  p Some text
```

Tags default to `div` when only a class or id is specified:

```hsml
.container
  .card
    .card-body Hello
```

### Classes

```hsml
h1.text-red.font-bold Hello
```

TailwindCSS arbitrary values are fully supported:

```hsml
.bg-[#1da1f2].lg:[&:nth-child(3)]:hover:underline
```

### IDs

```hsml
div#app
  h1#title Hello
```

### Attributes

```hsml
img(src="/photo.jpg" alt="A photo" width="300")
a(href="https://github.com" target="_blank") GitHub
button(disabled) Click me
```

Multiline attributes work too:

```hsml
img(
  src="/photo.jpg"
  alt="A photo"
  width="300"
  height="200"
)
```

### Text blocks

Use a trailing `.` to start a text block:

```hsml
p.
  This is a multi-line
  text block that preserves
  line breaks.
```

### Comments

```hsml
// Dev comment (excluded from HTML output)
//! Native comment (rendered as <!-- ... -->)
```

### Doctype

```hsml
doctype html
```

### Vue / Angular support

HSML supports framework-specific attribute syntax out of the box:

```hsml
// Vue
button(@click="handleClick" :class="dynamicClass" v-if="show") Click
template(#default)
  p Slot content

// Angular
button([disabled]="isDisabled" (click)="onClick()") Click
```

## Diagnostics

HSML provides helpful error messages with source context:

```log
error[E001]: Tag name must start with an ASCII letter
 --> example.hsml:1:1
  |
1 | 42div Hello
  | ^
```

And warnings for common mistakes:

```log
warning[W002]: Duplicate class 'foo'
 --> example.hsml:1:12
  |
1 | h1.foo.foo Hello
  |        ^
```

### Error & warning codes

| Code | Description                              |
| ---- | ---------------------------------------- |
| E001 | Tag name must start with an ASCII letter |
| E002 | Unclosed bracket                         |
| E003 | Unclosed parenthesis                     |
| E004 | Unclosed quote in attribute value        |
| E005 | Expected quoted attribute value          |
| E006 | Invalid attribute key                    |
| W001 | Duplicate id (only one per element)      |
| W002 | Duplicate class                          |
| W003 | Mixed tabs and spaces in indentation     |
| W004 | Duplicate attribute                      |

## Roadmap

- [x] Parser with TailwindCSS support
- [x] HTML compiler
- [x] CLI with `compile` command
- [x] WASM package for npm
- [x] Diagnostic system with errors and warnings
- [x] JSON diagnostic output (`--report-format json`)
- [x] `hsml check` — standalone linting command
- [x] Ignore support (`.gitignore`, `.hsmlignore`, `--ignore-pattern`)
- [ ] `hsml fmt` — code formatter
- [ ] `hsml parse` — AST output as JSON
- [ ] LSP server for editor integration
- [ ] GitHub/GitLab CI diagnostic formatters

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and commands.

## License

[MIT](LICENSE) — go wild.
