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
                HsmlNode::Tag(TagNode {
                    tag: String::from("h1"),
                    ids: vec![],
                    classes: Some(vec![ClassNode::new_without_location("text-red")]),
                    attributes: None,
                    text: Some(TextNode {
                        text: String::from("Vite CJS Faker Demo"),
                    }),
                    children: None,
                }),
                HsmlNode::Tag(TagNode {
                    tag: String::from("div"),
                    ids: vec![],
                    classes: Some(vec![ClassNode::new_without_location("card")]),
                    attributes: None,
                    text: None,
                    children: Some(vec![
                        HsmlNode::Tag(TagNode {
                            tag: String::from("div"),
                            ids: vec![],
                            classes: Some(vec![ClassNode::new_without_location("card__image")]),
                            attributes: None,
                            text: None,
                            children: Some(vec![HsmlNode::Tag(TagNode {
                                tag: String::from("img"),
                                ids: vec![],
                                classes: None,
                                attributes: Some(vec![
                                    HsmlNode::Attribute(AttributeNode::new_without_location(
                                        ":src",
                                        Some("natureImageUrl"),
                                    )),
                                    HsmlNode::Attribute(AttributeNode::new_without_location(
                                        ":alt",
                                        Some("'Background image for ' + fullName"),
                                    )),
                                ]),
                                text: None,
                                children: None,
                            })]),
                        }),
                        HsmlNode::Tag(TagNode {
                            tag: String::from("div"),
                            ids: vec![],
                            classes: Some(vec![ClassNode::new_without_location("card__profile")]),
                            attributes: None,
                            text: None,
                            children: Some(vec![HsmlNode::Tag(TagNode {
                                tag: String::from("img"),
                                ids: vec![],
                                classes: None,
                                attributes: Some(vec![
                                    HsmlNode::Attribute(AttributeNode::new_without_location(
                                        ":src",
                                        Some("avatarUrl"),
                                    )),
                                    HsmlNode::Attribute(AttributeNode::new_without_location(
                                        ":alt",
                                        Some("'Avatar image of ' + fullName"),
                                    )),
                                ]),
                                text: None,
                                children: None,
                            })]),
                        }),
                        HsmlNode::Tag(TagNode {
                            tag: String::from("div"),
                            ids: vec![],
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
                    ids: vec![],
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
                            ids: vec![],
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
                            ids: vec![],
                            classes: None,
                            attributes: Some(vec![
                                HsmlNode::Comment(CommentNode {
                                    text: String::from(" supports attribute inline comments"),
                                    is_dev: true,
                                }),
                                HsmlNode::Attribute(AttributeNode::new_without_location(
                                    "src",
                                    Some("/fancy-avatar.jpg"),
                                )),
                                HsmlNode::Attribute(AttributeNode::new_without_location(
                                    "alt",
                                    Some("Fancy Avatar"),
                                )),
                                HsmlNode::Comment(CommentNode {
                                    text: String::from(" the size of the image"),
                                    is_dev: true,
                                }),
                                HsmlNode::Attribute(AttributeNode::new_without_location(
                                    "width",
                                    Some("384"),
                                )),
                                HsmlNode::Attribute(AttributeNode::new_without_location(
                                    "height",
                                    Some("512"),
                                )),
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
                ids: vec![],
                classes: Some(vec![
                    ClassNode::new_without_location("rounded-full"),
                    ClassNode::new_without_location("mx-auto"),
                ]),
                attributes: Some(vec![
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
                text: None,
                children: None,
            })],
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
            nodes: vec![HsmlNode::Tag(TagNode {
                tag: String::from("div"),
                ids: vec![
                    IdNode::new_without_location("id1"),
                    IdNode::new_without_location("id2"),
                ],
                classes: None,
                attributes: None,
                text: None,
                children: None,
            })],
        }
    );

    assert_eq!(*rest.fragment(), "");
}
