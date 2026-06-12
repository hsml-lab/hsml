//! Lower-level token processing for Angular `@`-blocks.

/// Find the byte length of a balanced parenthesized head `(...)`, tracking
/// nesting depth and skipping quoted substrings (so parentheses inside string
/// literals don't affect the balance). The input must start with `(`. The head
/// may span multiple lines. Returns the length including both parentheses, or
/// `None` if the parentheses are unbalanced.
pub(super) fn balanced_parens_len(s: &str) -> Option<usize> {
    let mut chars = s.char_indices();
    if chars.next()?.1 != '(' {
        return None;
    }

    let mut depth = 1usize;

    while let Some((index, c)) = chars.next() {
        match c {
            '\'' | '"' | '`' => {
                let quote = c;
                let mut is_escaped = false;
                let mut closed = false;

                for (_, qc) in chars.by_ref() {
                    if is_escaped {
                        is_escaped = false;
                        continue;
                    }
                    if qc == '\\' {
                        is_escaped = true;
                        continue;
                    }
                    if qc == quote {
                        closed = true;
                        break;
                    }
                }

                closed.then_some(())?;
            }
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + c.len_utf8());
                }
            }
            _ => {}
        }
    }

    None
}

/// Find the byte index of the top-level `;` that terminates an `@let` expression.
///
/// Quoted substrings (`'…'`, `"…"`, `` `…` ``) are skipped so a `;` inside a
/// string literal does not terminate the expression. The expression may span
/// multiple lines. Returns `None` if no unquoted `;` is found (an unterminated
/// `@let`) or a string literal is left open.
pub(super) fn find_let_expression_end(s: &str) -> Option<usize> {
    let mut chars = s.char_indices();

    while let Some((index, c)) = chars.next() {
        match c {
            '\'' | '"' | '`' => {
                let quote = c;
                let mut is_escaped = false;
                let mut closed = false;

                for (_, qc) in chars.by_ref() {
                    if is_escaped {
                        is_escaped = false;
                        continue;
                    }
                    if qc == '\\' {
                        is_escaped = true;
                        continue;
                    }
                    if qc == quote {
                        closed = true;
                        break;
                    }
                }

                // An unclosed string means the whole `@let` is unterminated.
                closed.then_some(())?;
            }
            ';' => return Some(index),
            _ => {}
        }
    }

    None
}
