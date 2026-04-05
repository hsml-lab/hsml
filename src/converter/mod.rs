mod emitter;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::RcDom;

/// Convert an HTML string to HSML source.
pub fn convert(html: &str) -> Result<String, String> {
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .map_err(|e| format!("HTML parse error: {e}"))?;

    Ok(emitter::emit(&dom))
}

#[cfg(test)]
mod tests;
