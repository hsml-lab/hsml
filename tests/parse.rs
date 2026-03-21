use nom::error::ErrorKind;

use hsml::parser::{
    HsmlNode, RootNode, Span,
    attribute::node::AttributeNode,
    class::node::ClassNode,
    comment::node::CommentNode,
    error::{ErrorCode, Severity},
    parse::parse,
    tag::node::TagNode,
    text::node::TextNode,
};

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

    assert_eq!(
        root_node,
        RootNode {
            nodes: vec![
                HsmlNode::Tag(TagNode {
                    tag: String::from("h1"),
                    id: None,
                    classes: Some(vec![ClassNode::new_without_location("text-red")]),
                    attributes: None,
                    text: Some(TextNode {
                        text: String::from("Vite CJS Faker Demo"),
                    }),
                    children: None,
                }),
                HsmlNode::Tag(TagNode {
                    tag: String::from("div"),
                    id: None,
                    classes: Some(vec![ClassNode::new_without_location("card")]),
                    attributes: None,
                    text: None,
                    children: Some(vec![
                        HsmlNode::Tag(TagNode {
                            tag: String::from("div"),
                            id: None,
                            classes: Some(vec![ClassNode::new_without_location("card__image")]),
                            attributes: None,
                            text: None,
                            children: Some(vec![HsmlNode::Tag(TagNode {
                                tag: String::from("img"),
                                id: None,
                                classes: None,
                                attributes: Some(vec![
                                    HsmlNode::Attribute(AttributeNode {
                                        key: String::from(":src"),
                                        value: Some(String::from("natureImageUrl")),
                                    }),
                                    HsmlNode::Attribute(AttributeNode {
                                        key: String::from(":alt"),
                                        value: Some(String::from(
                                            "'Background image for ' + fullName"
                                        )),
                                    }),
                                ]),
                                text: None,
                                children: None,
                            })]),
                        }),
                        HsmlNode::Tag(TagNode {
                            tag: String::from("div"),
                            id: None,
                            classes: Some(vec![ClassNode::new_without_location("card__profile")]),
                            attributes: None,
                            text: None,
                            children: Some(vec![HsmlNode::Tag(TagNode {
                                tag: String::from("img"),
                                id: None,
                                classes: None,
                                attributes: Some(vec![
                                    HsmlNode::Attribute(AttributeNode {
                                        key: String::from(":src"),
                                        value: Some(String::from("avatarUrl")),
                                    }),
                                    HsmlNode::Attribute(AttributeNode {
                                        key: String::from(":alt"),
                                        value: Some(String::from("'Avatar image of ' + fullName")),
                                    }),
                                ]),
                                text: None,
                                children: None,
                            })]),
                        }),
                        HsmlNode::Tag(TagNode {
                            tag: String::from("div"),
                            id: None,
                            classes: Some(vec![ClassNode::new_without_location("card__body")]),
                            attributes: None,
                            text: Some(TextNode {
                                text: String::from("{{ fullName }}"),
                            }),
                            children: None,
                        })
                    ]),
                }),
            ],
        }
    );

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

    assert_eq!(
        root_node,
        RootNode {
            nodes: vec![
                HsmlNode::Comment(CommentNode {
                    text: String::from(" this is a root dev comment"),
                    is_dev: true,
                }),
                HsmlNode::Comment(CommentNode {
                    text: String::from(" this is a root native comment (will get rendered)"),
                    is_dev: false,
                }),
                HsmlNode::Tag(TagNode {
                    tag: String::from("div"),
                    id: None,
                    classes: None,
                    attributes: None,
                    text: None,
                    children: Some(vec![
                        HsmlNode::Comment(CommentNode {
                            text: String::from(" this is a child comment"),
                            is_dev: true,
                        }),
                        HsmlNode::Tag(TagNode {
                            tag: String::from("p"),
                            id: None,
                            classes: None,
                            attributes: None,
                            text: Some(TextNode {
                                text: String::from("another tag")
                            }),
                            children: None,
                        }),
                        HsmlNode::Comment(CommentNode {
                            text: String::from(" this is a child comment that gets rendered"),
                            is_dev: false,
                        }),
                        HsmlNode::Tag(TagNode {
                            tag: String::from("img"),
                            id: None,
                            classes: None,
                            attributes: Some(vec![
                                HsmlNode::Comment(CommentNode {
                                    text: String::from(" supports attribute inline comments"),
                                    is_dev: true,
                                }),
                                HsmlNode::Attribute(AttributeNode {
                                    key: String::from("src"),
                                    value: Some(String::from("/fancy-avatar.jpg")),
                                }),
                                HsmlNode::Attribute(AttributeNode {
                                    key: String::from("alt"),
                                    value: Some(String::from("Fancy Avatar")),
                                }),
                                HsmlNode::Comment(CommentNode {
                                    text: String::from(" the size of the image"),
                                    is_dev: true,
                                }),
                                HsmlNode::Attribute(AttributeNode {
                                    key: String::from("width"),
                                    value: Some(String::from("384")),
                                }),
                                HsmlNode::Attribute(AttributeNode {
                                    key: String::from("height"),
                                    value: Some(String::from("512")),
                                }),
                            ]),
                            text: None,
                            children: None,
                        }),
                    ])
                })
            ]
        }
    );

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

    assert_eq!(
        root_node,
        RootNode {
            nodes: vec![HsmlNode::Tag(TagNode {
                tag: String::from("img"),
                id: None,
                classes: Some(vec![
                    ClassNode::new_without_location("rounded-full"),
                    ClassNode::new_without_location("mx-auto"),
                ]),
                attributes: Some(vec![
                    HsmlNode::Attribute(AttributeNode {
                        key: String::from("src"),
                        value: Some(String::from("/fancy-avatar.jpg")),
                    }),
                    HsmlNode::Attribute(AttributeNode {
                        key: String::from("alt"),
                        value: Some(String::from("A fancy avatar")),
                    }),
                    HsmlNode::Attribute(AttributeNode {
                        key: String::from("width"),
                        value: Some(String::from("384")),
                    }),
                    HsmlNode::Attribute(AttributeNode {
                        key: String::from("height"),
                        value: Some(String::from("512")),
                    }),
                ]),
                text: None,
                children: None,
            })],
        }
    );

    assert_eq!(*rest.fragment(), "");
}

// Negative tests

#[test]
fn it_should_not_parse_tag_with_multiple_ids() {
    let input = r#"div#id1#id2"#;

    let result = parse(Span::new(input));
    assert!(result.is_err());
    if let Err(nom::Err::Failure(err)) = result {
        assert_eq!(*err.span.fragment(), "#id2");
        assert_eq!(err.kind, ErrorKind::Fail);
        assert_eq!(
            err.message.as_deref(),
            Some("Duplicate attribute 'id' is not allowed")
        );
        assert_eq!(err.error_code, Some(ErrorCode::DuplicateId));
        assert_eq!(err.code(), Some(ErrorCode::DuplicateId.code()));
        assert_eq!(err.severity, Severity::Error);
        assert_eq!(err.line(), 1);
        assert_eq!(err.column(), 8);
    } else {
        panic!("Expected Failure error");
    }
}
