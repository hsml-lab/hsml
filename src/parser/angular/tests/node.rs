use crate::common::Position;
use crate::parser::HsmlProcessContext;
use crate::parser::Span;
use crate::parser::angular::node::{
    AngularNode, BoundaryNode, CaseNode, CatchBlock, DefaultBranch, DeferBlock, DeferNode,
    ElseIfBranch, ForNode, IfNode, LetNode, SwitchNode, angular_node, boundary_node, defer_node,
    for_node, if_node, let_node, switch_node,
};
use crate::parser::tag::node::TagNode;
use crate::parser::text::node::TextNode;
use crate::parser::{HsmlNode, parse::parse};

#[test]
fn it_should_parse_single_line_let() {
    let (rest, node) = let_node(Span::new(
        "@let fullName = user.firstName + ' ' + user.lastName;\n",
    ))
    .unwrap();

    assert_eq!(
        node,
        LetNode::new_without_location("fullName", "user.firstName + ' ' + user.lastName")
    );
    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_parse_let_with_async_pipe() {
    let (rest, node) = let_node(Span::new("@let user = user$ | async;\n")).unwrap();

    assert_eq!(node, LetNode::new_without_location("user", "user$ | async"));
    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_not_terminate_on_semicolon_inside_a_string() {
    let (rest, node) = let_node(Span::new("@let sep = 'a;b';\n")).unwrap();

    assert_eq!(node, LetNode::new_without_location("sep", "'a;b'"));
    assert_eq!(*rest.fragment(), "\n");
}

#[test]
fn it_should_parse_a_multi_line_expression() {
    let input = concat!(
        "@let total = items\n",
        "  .filter((i) => i.active)\n",
        "  .reduce((sum, i) => sum + i.price, 0);\n",
        "h1 done\n",
    );

    let (rest, node) = let_node(Span::new(input)).unwrap();

    assert_eq!(
        node,
        LetNode::new_without_location(
            "total",
            "items\n  .filter((i) => i.active)\n  .reduce((sum, i) => sum + i.price, 0)"
        )
    );
    assert_eq!(*rest.fragment(), "\nh1 done\n");
}

#[test]
fn it_should_track_let_location() {
    let (_, node) = let_node(Span::new("@let x = 5;\n")).unwrap();

    assert_eq!(node.location.start, Position { line: 1, column: 1 });
    // End is one past the terminating ';'.
    assert_eq!(
        node.location.end,
        Position {
            line: 1,
            column: 12
        }
    );
}

#[test]
fn it_should_error_on_unterminated_let() {
    assert!(let_node(Span::new("@let x = 5\n")).is_err());
}

#[test]
fn it_should_error_when_at_keyword_is_not_let() {
    assert!(let_node(Span::new("@if (cond)\n")).is_err());
}

#[test]
fn it_should_dispatch_let_via_angular_node() {
    let mut context = HsmlProcessContext::default();
    let (_, node) = angular_node(Span::new("@let x = 5;\n"), &mut context).unwrap();

    assert_eq!(
        node,
        AngularNode::Let(LetNode::new_without_location("x", "5"))
    );
}

/// Build a `p <text>` tag node, the common body element used in these tests.
fn p(text: &str) -> HsmlNode {
    HsmlNode::Tag(TagNode::without_location(
        "p",
        vec![],
        None,
        None,
        Some(TextNode {
            text: text.to_string(),
            is_block: false,
        }),
        None,
    ))
}

#[test]
fn it_should_parse_if_with_then_branch_only() {
    let mut context = HsmlProcessContext::default();
    let (rest, node) = if_node(
        Span::new("@if (user.isAdmin)\n  p Admin\nh1 after\n"),
        &mut context,
    )
    .unwrap();

    assert_eq!(
        node,
        IfNode::new_without_location("user.isAdmin", vec![p("Admin")], vec![], None)
    );
    assert_eq!(*rest.fragment(), "\nh1 after\n");
}

#[test]
fn it_should_parse_if_else_chain() {
    let mut context = HsmlProcessContext::default();
    let input = "@if (a)\n  p A\n@else if (b)\n  p B\n@else\n  p C\n";
    let (_, node) = if_node(Span::new(input), &mut context).unwrap();

    assert_eq!(
        node,
        IfNode::new_without_location(
            "a",
            vec![p("A")],
            vec![ElseIfBranch {
                condition: "b".to_string(),
                body: vec![p("B")],
            }],
            Some(vec![p("C")]),
        )
    );
}

#[test]
fn it_should_parse_if_with_explicit_empty_body() {
    let mut context = HsmlProcessContext::default();
    let (rest, node) = if_node(Span::new("@if (hidden) {}\nh1 after\n"), &mut context).unwrap();

    assert_eq!(
        node,
        IfNode::new_without_location("hidden", vec![], vec![], None)
    );
    assert_eq!(*rest.fragment(), "\nh1 after\n");
}

#[test]
fn it_should_capture_condition_with_nested_parens_and_alias() {
    let mut context = HsmlProcessContext::default();
    let (_, node) = if_node(
        Span::new("@if ((a || b) && f(c); as result)\n  p ok\n"),
        &mut context,
    )
    .unwrap();

    assert_eq!(
        node,
        IfNode::new_without_location("(a || b) && f(c); as result", vec![p("ok")], vec![], None)
    );
}

#[test]
fn it_should_error_on_a_bare_if_without_a_body() {
    let mut context = HsmlProcessContext::default();
    // Dedent immediately after the head — no children and no `{}`.
    assert!(if_node(Span::new("@if (a)\nh1 sibling\n"), &mut context).is_err());
}

#[test]
fn it_should_treat_consecutive_if_blocks_as_siblings() {
    // Two root-level `@if` blocks must not be joined; the second is a separate node.
    let (rest, root) = parse(Span::new("@if (a)\n  p A\n@if (b)\n  p B\n")).unwrap();

    assert_eq!(root.nodes.len(), 2);
    assert!(matches!(
        root.nodes[0],
        HsmlNode::Angular(AngularNode::If(_))
    ));
    assert!(matches!(
        root.nodes[1],
        HsmlNode::Angular(AngularNode::If(_))
    ));
    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_error_on_orphan_else() {
    let mut context = HsmlProcessContext::default();
    assert!(angular_node(Span::new("@else\n  p x\n"), &mut context).is_err());
}

#[test]
fn it_should_parse_for_with_empty_block() {
    let mut context = HsmlProcessContext::default();
    let input = "@for (item of items; track item.id)\n  p item\n@empty\n  p none\n";
    let (_, node) = for_node(Span::new(input), &mut context).unwrap();

    assert_eq!(
        node,
        ForNode::new_without_location(
            "item of items; track item.id",
            vec![p("item")],
            Some(vec![p("none")]),
        )
    );
}

#[test]
fn it_should_parse_for_without_empty_block() {
    let mut context = HsmlProcessContext::default();
    let (rest, node) = for_node(
        Span::new("@for (x of xs; track x)\n  p x\nh1 after\n"),
        &mut context,
    )
    .unwrap();

    assert_eq!(
        node,
        ForNode::new_without_location("x of xs; track x", vec![p("x")], None)
    );
    assert_eq!(*rest.fragment(), "\nh1 after\n");
}

#[test]
fn it_should_capture_for_header_with_nested_parens_and_aliases() {
    let mut context = HsmlProcessContext::default();
    let (_, node) = for_node(
        Span::new("@for (x of xs; track fn(x, $index); let i = $index, e = $even)\n  p x\n"),
        &mut context,
    )
    .unwrap();

    assert_eq!(
        node,
        ForNode::new_without_location(
            "x of xs; track fn(x, $index); let i = $index, e = $even",
            vec![p("x")],
            None,
        )
    );
}

#[test]
fn it_should_error_on_bare_for_without_a_body() {
    let mut context = HsmlProcessContext::default();
    assert!(
        for_node(
            Span::new("@for (x of xs; track x)\nh1 sibling\n"),
            &mut context
        )
        .is_err()
    );
}

#[test]
fn it_should_error_on_orphan_empty() {
    let mut context = HsmlProcessContext::default();
    assert!(angular_node(Span::new("@empty\n  p x\n"), &mut context).is_err());
}

#[test]
fn it_should_dispatch_for_via_angular_node() {
    let mut context = HsmlProcessContext::default();
    let (_, node) =
        angular_node(Span::new("@for (x of xs; track x)\n  p x\n"), &mut context).unwrap();

    assert!(matches!(node, AngularNode::For(_)));
}

#[test]
fn it_should_parse_switch_with_cases_and_default() {
    let mut context = HsmlProcessContext::default();
    let input = "@switch (status)\n  @case (\"active\")\n    p Active\n  @default\n    p Unknown\n";
    let (_, node) = switch_node(Span::new(input), &mut context).unwrap();

    assert_eq!(
        node,
        SwitchNode::new_without_location(
            "status",
            vec![CaseNode {
                values: vec!["\"active\"".to_string()],
                body: vec![p("Active")],
            }],
            Some(DefaultBranch::Block(vec![p("Unknown")])),
        )
    );
}

#[test]
fn it_should_stack_consecutive_bare_cases() {
    let mut context = HsmlProcessContext::default();
    let input = "@switch (s)\n  @case (a)\n  @case (b)\n    p AB\n";
    let (_, node) = switch_node(Span::new(input), &mut context).unwrap();

    assert_eq!(
        node,
        SwitchNode::new_without_location(
            "s",
            vec![CaseNode {
                values: vec!["a".to_string(), "b".to_string()],
                body: vec![p("AB")],
            }],
            None,
        )
    );
}

#[test]
fn it_should_parse_an_explicit_empty_case() {
    let mut context = HsmlProcessContext::default();
    let (_, node) = switch_node(Span::new("@switch (s)\n  @case (a) {}\n"), &mut context).unwrap();

    assert_eq!(
        node,
        SwitchNode::new_without_location(
            "s",
            vec![CaseNode {
                values: vec!["a".to_string()],
                body: vec![],
            }],
            None,
        )
    );
}

#[test]
fn it_should_parse_default_never() {
    let mut context = HsmlProcessContext::default();
    let (_, node) = switch_node(
        Span::new("@switch (s)\n  @case (a)\n    p A\n  @default never;\n"),
        &mut context,
    )
    .unwrap();

    assert_eq!(node.default, Some(DefaultBranch::Never(None)));
}

#[test]
fn it_should_parse_default_never_with_expression() {
    let mut context = HsmlProcessContext::default();
    let (_, node) = switch_node(
        Span::new("@switch (s)\n  @case (a)\n    p A\n  @default never(state);\n"),
        &mut context,
    )
    .unwrap();

    assert_eq!(
        node.default,
        Some(DefaultBranch::Never(Some("state".to_string())))
    );
}

#[test]
fn it_should_error_on_a_trailing_bare_case() {
    let mut context = HsmlProcessContext::default();
    assert!(
        switch_node(
            Span::new("@switch (s)\n  @case (a)\n  @default\n    p x\n"),
            &mut context
        )
        .is_err()
    );
}

#[test]
fn it_should_error_on_a_non_case_child_in_switch() {
    let mut context = HsmlProcessContext::default();
    assert!(switch_node(Span::new("@switch (s)\n  p oops\n"), &mut context).is_err());
}

#[test]
fn it_should_dispatch_switch_via_angular_node() {
    let mut context = HsmlProcessContext::default();
    let (_, node) = angular_node(
        Span::new("@switch (s)\n  @case (a)\n    p A\n"),
        &mut context,
    )
    .unwrap();

    assert!(matches!(node, AngularNode::Switch(_)));
}

#[test]
fn it_should_parse_defer_with_all_sub_blocks() {
    let mut context = HsmlProcessContext::default();
    let input = "@defer (on viewport)\n  p main\n@placeholder (minimum 500ms)\n  p ph\n@loading\n  p ld\n@error\n  p err\n";
    let (_, node) = defer_node(Span::new(input), &mut context).unwrap();

    assert_eq!(
        node,
        DeferNode::new_without_location(
            Some("on viewport".to_string()),
            vec![p("main")],
            Some(DeferBlock {
                params: Some("minimum 500ms".to_string()),
                body: vec![p("ph")],
            }),
            Some(DeferBlock {
                params: None,
                body: vec![p("ld")],
            }),
            Some(vec![p("err")]),
        )
    );
}

#[test]
fn it_should_parse_defer_without_a_head() {
    let mut context = HsmlProcessContext::default();
    let (rest, node) = defer_node(Span::new("@defer\n  p x\nh1 after\n"), &mut context).unwrap();

    assert_eq!(
        node,
        DeferNode::new_without_location(None, vec![p("x")], None, None, None)
    );
    assert_eq!(*rest.fragment(), "\nh1 after\n");
}

#[test]
fn it_should_error_on_duplicate_defer_placeholder() {
    let mut context = HsmlProcessContext::default();
    let input = "@defer\n  p x\n@placeholder\n  p a\n@placeholder\n  p b\n";
    assert!(defer_node(Span::new(input), &mut context).is_err());
}

#[test]
fn it_should_dispatch_defer_via_angular_node() {
    let mut context = HsmlProcessContext::default();
    let (_, node) = angular_node(Span::new("@defer\n  p x\n"), &mut context).unwrap();

    assert!(matches!(node, AngularNode::Defer(_)));
}

#[test]
fn it_should_parse_boundary_with_catch() {
    let mut context = HsmlProcessContext::default();
    let input = "@boundary\n  p main\n@catch (error)\n  p fallback\n";
    let (_, node) = boundary_node(Span::new(input), &mut context).unwrap();

    assert_eq!(
        node,
        BoundaryNode::new_without_location(
            vec![p("main")],
            Some(CatchBlock {
                binding: Some("error".to_string()),
                body: vec![p("fallback")],
            }),
        )
    );
}

#[test]
fn it_should_parse_boundary_without_catch() {
    let mut context = HsmlProcessContext::default();
    let (rest, node) =
        boundary_node(Span::new("@boundary\n  p x\nh1 after\n"), &mut context).unwrap();

    assert_eq!(node, BoundaryNode::new_without_location(vec![p("x")], None));
    assert_eq!(*rest.fragment(), "\nh1 after\n");
}

#[test]
fn it_should_dispatch_boundary_via_angular_node() {
    let mut context = HsmlProcessContext::default();
    let (_, node) = angular_node(Span::new("@boundary\n  p x\n"), &mut context).unwrap();

    assert!(matches!(node, AngularNode::Boundary(_)));
}
