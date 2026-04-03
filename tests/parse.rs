use hsml::parser::{
    HsmlNode, RootNode, Span, attribute::node::AttributeNode, class::node::ClassNode,
    comment::node::CommentNode, id::node::IdNode, parse::parse, tag::node::TagNode,
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
                HsmlNode::Tag(TagNode::without_location(
                    "h1",
                    vec![],
                    Some(vec![ClassNode::new_without_location("text-red")]),
                    None,
                    Some(TextNode {
                        text: String::from("Vite CJS Faker Demo"),
                    }),
                    None,
                )),
                HsmlNode::Tag(TagNode::without_location(
                    "div",
                    vec![],
                    Some(vec![ClassNode::new_without_location("card")]),
                    None,
                    None,
                    Some(vec![
                        HsmlNode::Tag(TagNode::without_location(
                            "div",
                            vec![],
                            Some(vec![ClassNode::new_without_location("card__image")]),
                            None,
                            None,
                            Some(vec![HsmlNode::Tag(TagNode::without_location(
                                "img",
                                vec![],
                                None,
                                Some(vec![
                                    HsmlNode::Attribute(AttributeNode::new_without_location(
                                        ":src",
                                        Some("natureImageUrl"),
                                    )),
                                    HsmlNode::Attribute(AttributeNode::new_without_location(
                                        ":alt",
                                        Some("'Background image for ' + fullName"),
                                    )),
                                ]),
                                None,
                                None,
                            ))]),
                        )),
                        HsmlNode::Tag(TagNode::without_location(
                            "div",
                            vec![],
                            Some(vec![ClassNode::new_without_location("card__profile")]),
                            None,
                            None,
                            Some(vec![HsmlNode::Tag(TagNode::without_location(
                                "img",
                                vec![],
                                None,
                                Some(vec![
                                    HsmlNode::Attribute(AttributeNode::new_without_location(
                                        ":src",
                                        Some("avatarUrl"),
                                    )),
                                    HsmlNode::Attribute(AttributeNode::new_without_location(
                                        ":alt",
                                        Some("'Avatar image of ' + fullName"),
                                    )),
                                ]),
                                None,
                                None,
                            ))]),
                        )),
                        HsmlNode::Tag(TagNode::without_location(
                            "div",
                            vec![],
                            Some(vec![ClassNode::new_without_location("card__body")]),
                            None,
                            Some(TextNode {
                                text: String::from("{{ fullName }}"),
                            }),
                            None,
                        ))
                    ]),
                )),
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
                HsmlNode::Comment(CommentNode::new_without_location(
                    " this is a root dev comment",
                    true
                )),
                HsmlNode::Comment(CommentNode::new_without_location(
                    " this is a root native comment (will get rendered)",
                    false
                )),
                HsmlNode::Tag(TagNode::without_location(
                    "div",
                    vec![],
                    None,
                    None,
                    None,
                    Some(vec![
                        HsmlNode::Comment(CommentNode::new_without_location(
                            " this is a child comment",
                            true
                        )),
                        HsmlNode::Tag(TagNode::without_location(
                            "p",
                            vec![],
                            None,
                            None,
                            Some(TextNode {
                                text: String::from("another tag")
                            }),
                            None,
                        )),
                        HsmlNode::Comment(CommentNode::new_without_location(
                            " this is a child comment that gets rendered",
                            false
                        )),
                        HsmlNode::Tag(TagNode::without_location(
                            "img",
                            vec![],
                            None,
                            Some(vec![
                                HsmlNode::Comment(CommentNode::new_without_location(
                                    " supports attribute inline comments",
                                    true
                                )),
                                HsmlNode::Attribute(AttributeNode::new_without_location(
                                    "src",
                                    Some("/fancy-avatar.jpg"),
                                )),
                                HsmlNode::Attribute(AttributeNode::new_without_location(
                                    "alt",
                                    Some("Fancy Avatar"),
                                )),
                                HsmlNode::Comment(CommentNode::new_without_location(
                                    " the size of the image",
                                    true
                                )),
                                HsmlNode::Attribute(AttributeNode::new_without_location(
                                    "width",
                                    Some("384"),
                                )),
                                HsmlNode::Attribute(AttributeNode::new_without_location(
                                    "height",
                                    Some("512"),
                                )),
                            ]),
                            None,
                            None,
                        )),
                    ])
                ))
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
            nodes: vec![HsmlNode::Tag(TagNode::without_location(
                "img",
                vec![],
                Some(vec![
                    ClassNode::new_without_location("rounded-full"),
                    ClassNode::new_without_location("mx-auto"),
                ]),
                Some(vec![
                    HsmlNode::Attribute(AttributeNode::new_without_location(
                        "src",
                        Some("/fancy-avatar.jpg"),
                    )),
                    HsmlNode::Attribute(AttributeNode::new_without_location(
                        "alt",
                        Some("A fancy avatar"),
                    )),
                    HsmlNode::Attribute(AttributeNode::new_without_location("width", Some("384"),)),
                    HsmlNode::Attribute(
                        AttributeNode::new_without_location("height", Some("512"),)
                    ),
                ]),
                None,
                None,
            ))],
        }
    );

    assert_eq!(*rest.fragment(), "");
}

// Negative tests

#[test]
fn it_should_parse_tag_with_multiple_ids() {
    let input = "div#id1#id2\n";

    let (rest, root_node) = parse(Span::new(input)).unwrap();

    assert_eq!(
        root_node,
        RootNode {
            nodes: vec![HsmlNode::Tag(TagNode::without_location(
                "div",
                vec![
                    IdNode::new_without_location("id1"),
                    IdNode::new_without_location("id2"),
                ],
                None,
                None,
                None,
                None,
            ))],
        }
    );

    assert_eq!(*rest.fragment(), "");
}
