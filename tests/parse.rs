use hsml::parser::{Span, parse::parse};

// AST structure is captured via JSON snapshots. The `.**.location` redaction
// drops source locations so these tests stay focused on structure — location
// correctness is asserted separately in the validator tests.
macro_rules! assert_ast_snapshot {
    ($ast:expr) => {
        insta::assert_json_snapshot!($ast, { ".**.location" => "[location]" })
    };
}

#[test]
fn it_should_parse() {
    let input = r#"h1.text-red Vite CJS Faker Demo
.card
  .card__image
    img(:src="natureImageUrl" :alt="'Background image for ' + fullName")
  .card__profile
    img(:src="avatarUrl" :alt="'Avatar image of ' + fullName")
  .card__body {{ fullName }}
"#;

    let (rest, root_node) = parse(Span::new(input)).unwrap();

    assert_ast_snapshot!(root_node);
    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_parse_with_comments() {
    let input = r#"// this is a root dev comment
//! this is a root native comment (will get rendered)
div
    // this is a child comment
    p another tag
    //! this is a child comment that gets rendered
    img(
        // supports attribute inline comments
        src="/fancy-avatar.jpg"
        alt="Fancy Avatar"
        // the size of the image
        width="384"
        height="512"
    )
"#;

    let (rest, root_node) = parse(Span::new(input)).unwrap();

    assert_ast_snapshot!(root_node);
    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_parse_wrapped_attributes() {
    let input = r#"img.rounded-full.mx-auto(
    src="/fancy-avatar.jpg"
    alt="A fancy avatar"
    width="384"
    height="512"
)
"#;

    let (rest, root_node) = parse(Span::new(input)).unwrap();

    assert_ast_snapshot!(root_node);
    assert_eq!(*rest.fragment(), "");
}

// Negative tests

#[test]
fn it_should_parse_tag_with_multiple_ids() {
    let input = "div#id1#id2\n";

    let (rest, root_node) = parse(Span::new(input)).unwrap();

    assert_ast_snapshot!(root_node);
    assert_eq!(*rest.fragment(), "");
}
