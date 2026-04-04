use crate::formatter::{FormatOptions, format};
use crate::parser::{Span, parse::parse};

fn fmt(input: &str) -> String {
    let (_, ast) = parse(Span::new(input)).unwrap();
    format(&ast, &FormatOptions::default())
}

#[test]
fn it_should_format_simple_tag_with_text() {
    assert_eq!(fmt("h1 Hello World\n"), "h1 Hello World\n");
}

#[test]
fn it_should_format_tag_with_class() {
    assert_eq!(fmt("h1.title Hello\n"), "h1.title Hello\n");
}

#[test]
fn it_should_format_implicit_div_with_class() {
    assert_eq!(fmt(".container\n"), ".container\n");
}

#[test]
fn it_should_format_implicit_div_with_id() {
    assert_eq!(fmt("#app\n"), "#app\n");
}

#[test]
fn it_should_format_tag_with_id_and_classes() {
    assert_eq!(
        fmt("div#app.foo.bar\n"),
        "#app.foo.bar\n" // implicit div
    );
}

#[test]
fn it_should_format_nested_tags() {
    assert_eq!(
        fmt("div\n  h1 Hello\n  p World\n"),
        "div\n  h1 Hello\n  p World\n"
    );
}

#[test]
fn it_should_format_deeply_nested() {
    assert_eq!(
        fmt("div\n  section\n    p Hello\n"),
        "div\n  section\n    p Hello\n"
    );
}

#[test]
fn it_should_format_attributes_single_line() {
    assert_eq!(
        fmt("img(src=\"/photo.jpg\" alt=\"Photo\")\n"),
        "img(src=\"/photo.jpg\", alt=\"Photo\")\n"
    );
}

#[test]
fn it_should_format_attributes_with_commas_normalized() {
    // Input has commas — output should normalize to comma-separated
    assert_eq!(
        fmt("div(class=\"a\",class=\"b\")\n"),
        "div(class=\"a\", class=\"b\")\n"
    );
}

#[test]
fn it_should_format_boolean_attribute() {
    assert_eq!(fmt("button(disabled)\n"), "button(disabled)\n");
}

#[test]
fn it_should_wrap_long_attributes_to_multiline() {
    let input = "div(class=\"very-long-class-name\" data-value=\"some-long-value\" aria-label=\"accessible label\" role=\"button\")\n";
    let expected = "\
div(
  class=\"very-long-class-name\",
  data-value=\"some-long-value\",
  aria-label=\"accessible label\",
  role=\"button\"
)
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_keep_trailing_comment_on_same_line() {
    // The formatter normalizes trailing commas after inline comments:
    // `alt="", // this is empty,` → `alt="", // this is empty`
    let input = "\
img(
  src=\"/photo.jpg\",
  alt=\"\", // this is empty,
  width=\"384\"
)
";
    let expected = "\
img(
  src=\"/photo.jpg\",
  alt=\"\", // this is empty
  width=\"384\"
)
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_keep_standalone_comment_on_separate_line() {
    let input = "\
img(
  // a comment,
  src=\"/photo.jpg\",
  alt=\"\",
  // another comment,
  width=\"384\"
)
";
    let expected = "\
img(
  // a comment
  src=\"/photo.jpg\",
  alt=\"\",
  // another comment
  width=\"384\"
)
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_format_text_block() {
    assert_eq!(
        fmt("p.\n  Line one\n  Line two\n"),
        "p.\n  Line one\n  Line two\n"
    );
}

#[test]
fn it_should_wrap_text_block_at_print_width() {
    let input = "\
.text-blue.
  Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum.
  Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.
  At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.
";
    let expected = "\
.text-blue.
  Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy
  eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam
  voluptua. At vero eos et accusam et justo duo dolores et ea rebum.
  Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit
  amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy
  eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam
  voluptua.
  At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd
  gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_expand_long_inline_text_to_block() {
    let input = "p Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor\n";
    let expected = "\
p.
  Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy
  eirmod tempor
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_not_expand_short_inline_text_to_block() {
    assert_eq!(fmt("p Hello World\n"), "p Hello World\n");
}

#[test]
fn it_should_not_collapse_short_text_block_to_inline() {
    assert_eq!(fmt("p.\n  Short\n"), "p.\n  Short\n");
}

#[test]
fn it_should_collapse_consecutive_blank_lines() {
    let input = "p.\n  Line one\n\n\n  Line two\n";
    let expected = "p.\n  Line one\n\n  Line two\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_preserve_blank_line_between_siblings() {
    let input = "\
div
  p A textline

  p.text-red Another textline
";
    assert_eq!(fmt(input), input);
}

#[test]
fn it_should_not_insert_blank_line_when_none_existed() {
    let input = "\
div
  p First
  p Second
";
    assert_eq!(fmt(input), input);
}

#[test]
fn it_should_remove_blank_line_between_parent_and_first_child() {
    let input = "\
figcaption.font-medium

  .text-sky Hello
";
    let expected = "\
figcaption.font-medium
  .text-sky Hello
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_preserve_blank_line_after_multiline_sibling() {
    let input = "\
figure
  img(src=\"/a-very-long-path/to/photo.jpg\", alt=\"A descriptive alt text for the photo\")

  .content Hello
";
    let expected = "\
figure
  img(
    src=\"/a-very-long-path/to/photo.jpg\",
    alt=\"A descriptive alt text for the photo\"
  )

  .content Hello
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_collapse_multiple_blank_lines_between_siblings() {
    let input = "\
div
  p First


  p Second
";
    let expected = "\
div
  p First

  p Second
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_preserve_blank_line_between_deep_siblings() {
    let input = "\
.card
  .card__image
    img(:src=\"url\")

  .card__profile
    img(:src=\"avatar\")
  .card__body Hello
";
    assert_eq!(fmt(input), input);
}

#[test]
fn it_should_preserve_blank_line_between_nested_nodes() {
    let input = "\
.card
     
  .card__image
    img(:src=\"url\")
    
  
  .card__profile

    img(:src=\"avatar\")
  .card__body Hello
";
    let expected = "\
.card
  .card__image
    img(:src=\"url\")

  .card__profile
    img(:src=\"avatar\")
  .card__body Hello
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_handle_file_ending_with_trailing_whitespace_no_newline() {
    // File ends with "    " (spaces, no newline) — parser should handle EOF gracefully
    let input = "div\n  p Hello\n    ";
    let expected = "div\n  p Hello\n";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_normalize_whitespace_and_preserve_blank_lines_in_complex_document() {
    let input = "\
figure.md:flex.bg-slate-100.rounded-xl.p-8.md:p-0.dark:bg-slate-800/10
  
  img.w-24.h-24.md:w-48.md:h-auto.md:rounded-none.rounded-full.mx-auto(
    // supports attribute inline comments
    src=\"/fancy-avatar.jpg\",
    alt=\"\", // this is empty 🤷
    width=\"384\",
    height=\"512\"
  )

  .pt-6.md:p-8.text-center.md:text-left.space-y-4
     blockquote(v-if=\"showBlockquote\")
      p.text-lg.font-medium.
        \"Tailwind CSS is the only framework that I've seen scale
        on large teams. It's easy to customize, adapts to any design,
        and the build size is tiny.\"
    
";
    let expected = "\
figure.md:flex.bg-slate-100.rounded-xl.p-8.md:p-0.dark:bg-slate-800/10
  img.w-24.h-24.md:w-48.md:h-auto.md:rounded-none.rounded-full.mx-auto(
    // supports attribute inline comments
    src=\"/fancy-avatar.jpg\",
    alt=\"\", // this is empty 🤷
    width=\"384\",
    height=\"512\"
  )

  .pt-6.md:p-8.text-center.md:text-left.space-y-4
    blockquote(v-if=\"showBlockquote\")
      p.text-lg.font-medium.
        \"Tailwind CSS is the only framework that I've seen scale
        on large teams. It's easy to customize, adapts to any design,
        and the build size is tiny.\"
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_strip_trailing_whitespace_from_blank_lines_in_text_blocks() {
    // Blank line with trailing spaces in text block should become empty
    let input = "\
p.
  Line one
  
  Line two
";
    let expected = "\
p.
  Line one

  Line two
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_not_add_blank_line_when_none_between_deep_siblings() {
    let input = "\
.card
  .card__image
    img(:src=\"url\")
  .card__profile
    img(:src=\"avatar\")
";
    assert_eq!(fmt(input), input);
}

#[test]
fn it_should_format_dev_comment() {
    assert_eq!(fmt("// hello\n"), "// hello\n");
}

#[test]
fn it_should_format_native_comment() {
    assert_eq!(fmt("//! hello\n"), "//! hello\n");
}

#[test]
fn it_should_format_doctype() {
    assert_eq!(fmt("doctype html\n"), "doctype html\n");
}

#[test]
fn it_should_normalize_indentation() {
    // Input uses 4 spaces — formatter normalizes to 2
    assert_eq!(fmt("div\n    h1 Hello\n"), "div\n  h1 Hello\n");
}

#[test]
fn it_should_ensure_trailing_newline() {
    // The parser requires a trailing newline, so we can only test preservation.
    // The formatter's trailing newline logic is a safety net for edge cases.
    let output = fmt("h1 Hello\n");
    assert!(output.ends_with('\n'));
    assert!(!output.ends_with("\n\n"));
}

#[test]
fn it_should_format_full_document() {
    let input = "\
doctype html
html
  head
    meta(charset=\"utf-8\")
    title My Page
  body
    .container
      h1.title Hello
      p Some text
";
    let expected = "\
doctype html
html
  head
    meta(charset=\"utf-8\")
    title My Page
  body
    .container
      h1.title Hello
      p Some text
";
    assert_eq!(fmt(input), expected);
}

#[test]
fn it_should_be_idempotent() {
    let input = "\
doctype html
html
  head
    meta(charset=\"utf-8\")
    title My Page
  body
    .container#app
      img.rounded(src=\"/photo.jpg\", alt=\"Photo\")

      p.text-gray Hello World
      p.text-lg.font-medium.
        Some long text that spans multiple lines in the source and will be
        wrapped by the formatter.
      // dev comment
      //! native comment
";
    let first = fmt(input);
    let second = fmt(&first);
    assert_eq!(first, second, "formatting should be idempotent");
}
