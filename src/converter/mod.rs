mod emitter;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::RcDom;

use crate::common::is_void_element;

/// Convert an HTML string to HSML source.
pub fn convert(html: &str) -> Result<String, String> {
    let preprocessed = expand_self_closing_tags(html);
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut preprocessed.as_bytes())
        .map_err(|e| format!("HTML parse error: {e}"))?;

    Ok(emitter::emit(&dom, &preprocessed))
}

/// Expand self-closing non-void elements (e.g. `<div />` → `<div></div>`).
///
/// HTML5 only allows void elements to be self-closing. html5ever follows the
/// spec and treats `<div />` as an opening `<div>` tag, which swallows all
/// subsequent siblings as children. This pre-processing step fixes that by
/// expanding such tags into proper open/close pairs before parsing.
fn expand_self_closing_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let bytes = html.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Skip HTML comments
        if bytes[i] == b'<' && html[i..].starts_with("<!--") {
            if let Some(end) = html[i + 4..].find("-->") {
                let comment_end = i + 4 + end + 3;
                result.push_str(&html[i..comment_end]);
                i = comment_end;
                continue;
            }
        }

        // Check for start of an opening tag
        if bytes[i] == b'<'
            && i + 1 < bytes.len()
            && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_')
        {
            let tag_start = i + 1;
            let mut j = tag_start;
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-' || bytes[j] == b'_')
            {
                j += 1;
            }
            let tag_name = &html[tag_start..j];

            // Scan through attributes to find > or />
            let mut in_quote = false;
            let mut quote_char = b'"';
            let mut found_end = false;
            while j < bytes.len() {
                if in_quote {
                    if bytes[j] == quote_char {
                        in_quote = false;
                    }
                } else {
                    match bytes[j] {
                        b'"' | b'\'' => {
                            in_quote = true;
                            quote_char = bytes[j];
                        }
                        b'/' if j + 1 < bytes.len() && bytes[j + 1] == b'>' => {
                            if !is_void_element(tag_name) {
                                // Non-void: <tag .../>  →  <tag ...></tag>
                                result.push_str(&html[i..j]);
                                result.push_str("></");
                                result.push_str(tag_name);
                                result.push('>');
                            } else {
                                // Void element: keep as-is
                                result.push_str(&html[i..j + 2]);
                            }
                            i = j + 2;
                            found_end = true;
                            break;
                        }
                        b'>' => {
                            result.push_str(&html[i..j + 1]);
                            i = j + 1;
                            found_end = true;
                            break;
                        }
                        _ => {}
                    }
                }
                j += 1;
            }
            if !found_end {
                result.push_str(&html[i..]);
                break;
            }
        } else {
            // Copy everything up to the next '<' in bulk
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            result.push_str(&html[start..i]);
        }
    }

    result
}

#[cfg(test)]
mod tests;
