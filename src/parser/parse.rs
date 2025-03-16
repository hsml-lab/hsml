use nom::{IResult, bytes::complete::take_till};

use super::{
    HsmlNode, HsmlProcessContext, RootNode,
    comment::node::{comment_dev_node, comment_native_node},
    tag::node::tag_node,
};

pub fn parse(input: &str) -> IResult<&str, RootNode> {
    let mut nodes: Vec<HsmlNode> = vec![];

    let mut context = HsmlProcessContext::default();

    let mut input = input;

    loop {
        // eat leading and trailing newlines and whitespace if there are any
        if let Ok((rest, taken)) =
            take_till::<_, &str, nom::error::Error<&str>>(|c: char| !c.is_whitespace())(input)
        {
            // take the leading spaces and tabs after the last newline as indentation
            context.indent_string = taken
                .chars()
                .rev()
                .take_while(|c| c.is_whitespace() && *c != '\n')
                .collect::<String>()
                .chars()
                .rev()
                .collect();

            input = rest;

            if input.is_empty() {
                break;
            }
        }

        if let Ok((rest, node)) = comment_native_node(input) {
            nodes.push(HsmlNode::Comment(node));
            input = rest;
            continue;
        }

        if let Ok((rest, node)) = comment_dev_node(input) {
            nodes.push(HsmlNode::Comment(node));
            input = rest;
            continue;
        }

        match tag_node(input, &mut context) {
            Ok((rest, node)) => {
                nodes.push(HsmlNode::Tag(node));
                input = rest;
                continue;
            }
            Err(e) => {
                return Err(e);
            }
        }

        // TODO @Shinigami92 2023-05-18: Add support for doctype node
    }

    Ok((input, RootNode { nodes }))
}

#[cfg(test)]
mod tests {
    use nom::error::{Error, ErrorKind};

    use crate::parser::{
        HsmlNode, RootNode, attribute::node::AttributeNode, class::node::ClassNode,
        comment::node::CommentNode, parse::parse, tag::node::TagNode, text::node::TextNode,
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

        let (input, root_node) = parse(input).unwrap();

        assert_eq!(
            root_node,
            RootNode {
                nodes: vec![
                    HsmlNode::Tag(TagNode {
                        tag: String::from("h1"),
                        id: None,
                        classes: Some(vec![ClassNode {
                            name: String::from("text-red")
                        }]),
                        attributes: None,
                        text: Some(TextNode {
                            text: String::from("Vite CJS Faker Demo"),
                        }),
                        children: None,
                    }),
                    HsmlNode::Tag(TagNode {
                        tag: String::from("div"),
                        id: None,
                        classes: Some(vec![ClassNode {
                            name: String::from("card"),
                        }]),
                        attributes: None,
                        text: None,
                        children: Some(vec![
                            HsmlNode::Tag(TagNode {
                                tag: String::from("div"),
                                id: None,
                                classes: Some(vec![ClassNode {
                                    name: String::from("card__image"),
                                }]),
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
                                classes: Some(vec![ClassNode {
                                    name: String::from("card__profile"),
                                }]),
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
                                            value: Some(String::from(
                                                "'Avatar image of ' + fullName"
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
                                classes: Some(vec![ClassNode {
                                    name: String::from("card__body"),
                                }]),
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

        assert_eq!(input, "");
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

        let (input, root_node) = parse(input).unwrap();

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

        assert_eq!(input, "");
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

        let (input, root_node) = parse(input).unwrap();

        assert_eq!(
            root_node,
            RootNode {
                nodes: vec![HsmlNode::Tag(TagNode {
                    tag: String::from("img"),
                    id: None,
                    classes: Some(vec![
                        ClassNode {
                            name: String::from("rounded-full"),
                        },
                        ClassNode {
                            name: String::from("mx-auto"),
                        },
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

        assert_eq!(input, "");
    }

    // Negative tests

    #[test]
    fn it_should_not_parse_tag_with_multiple_ids() {
        let input = r#"div#id1#id2"#;

        assert_eq!(
            Err(nom::Err::Failure(Error {
                input: "#id2",
                code: ErrorKind::Tag
            })),
            parse(input)
        );
    }
}
