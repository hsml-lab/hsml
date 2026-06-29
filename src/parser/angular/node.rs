use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, space0, space1};
use serde::Serialize;

use crate::common::{Location, Position};
use crate::parser::children::{advance_to_next_line, parse_children};
use crate::parser::{HsmlNode, HsmlProcessContext, HsmlResult, Span, advance, error::HsmlError};

use super::process::{balanced_parens_len, find_let_expression_end};

/// An Angular `@`-block node — the umbrella for Angular-specific block syntax
/// (control flow, deferrable views, declarations, error boundaries).
///
/// The Angular block constructs HSML understands: `@let`, `@if`, `@for`,
/// `@switch`, `@defer` and `@boundary`.
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AngularNode {
    /// An `@let name = expression;` local template variable declaration.
    Let(LetNode),
    /// An `@if (…) / @else if (…) / @else` conditional block.
    If(IfNode),
    /// A `@for (… ; track …)` loop with an optional `@empty` block.
    For(ForNode),
    /// A `@switch (…)` block with `@case` / `@default` branches.
    Switch(SwitchNode),
    /// A `@defer` deferrable view with optional `@placeholder` / `@loading` / `@error`.
    Defer(DeferNode),
    /// A `@boundary` error boundary with an optional `@catch (error)` block.
    Boundary(BoundaryNode),
}

impl AngularNode {
    /// The source location of this block, used by the formatter (blank-line
    /// preservation) and diagnostics.
    pub fn location(&self) -> &Location {
        match self {
            AngularNode::Let(node) => &node.location,
            AngularNode::If(node) => &node.location,
            AngularNode::For(node) => &node.location,
            AngularNode::Switch(node) => &node.location,
            AngularNode::Defer(node) => &node.location,
            AngularNode::Boundary(node) => &node.location,
        }
    }
}

/// An Angular `@let` declaration: `@let name = expression;`.
///
/// The expression is captured verbatim (HSML never evaluates it) from after the
/// `=` up to the terminating `;`, and may span multiple lines.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LetNode {
    pub name: String,
    pub expression: String,
    /// Source location spanning `@let` through the terminating `;`.
    pub location: Location,
}

// PartialEq excludes location so that tests comparing parsed ASTs
// don't need to specify exact location values.
impl PartialEq for LetNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.expression == other.expression
    }
}

impl LetNode {
    /// Create a LetNode without a meaningful source location.
    /// Useful in tests where location is not relevant.
    #[doc(hidden)]
    pub fn new_without_location(name: impl Into<String>, expression: impl Into<String>) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            name: name.into(),
            expression: expression.into(),
            location: Location {
                start: zero,
                end: zero,
            },
        }
    }
}

/// An `@if` conditional block with optional `@else if` / `@else` branches.
///
/// The condition is the raw text inside the `@if (...)` parentheses, captured
/// verbatim (it may include a `; as alias`) and emitted back unchanged.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IfNode {
    pub condition: String,
    pub then_branch: Vec<HsmlNode>,
    pub else_if_branches: Vec<ElseIfBranch>,
    pub else_branch: Option<Vec<HsmlNode>>,
    /// Source location spanning the whole `@if … (… @else)` chain.
    pub location: Location,
}

// PartialEq excludes location so that tests comparing parsed ASTs
// don't need to specify exact location values.
impl PartialEq for IfNode {
    fn eq(&self, other: &Self) -> bool {
        self.condition == other.condition
            && self.then_branch == other.then_branch
            && self.else_if_branches == other.else_if_branches
            && self.else_branch == other.else_branch
    }
}

impl IfNode {
    /// Create an IfNode without a meaningful source location (for tests).
    #[doc(hidden)]
    pub fn new_without_location(
        condition: impl Into<String>,
        then_branch: Vec<HsmlNode>,
        else_if_branches: Vec<ElseIfBranch>,
        else_branch: Option<Vec<HsmlNode>>,
    ) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            condition: condition.into(),
            then_branch,
            else_if_branches,
            else_branch,
            location: Location {
                start: zero,
                end: zero,
            },
        }
    }
}

/// A single `@else if (condition)` branch of an [`IfNode`].
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElseIfBranch {
    pub condition: String,
    pub body: Vec<HsmlNode>,
}

/// A `@for (… ; track …)` loop with an optional `@empty` block.
///
/// The header is the raw text inside `@for (...)`, captured verbatim (it includes
/// the mandatory `track` clause and any `let` aliases) and emitted back unchanged.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForNode {
    pub expression: String,
    pub body: Vec<HsmlNode>,
    pub empty_branch: Option<Vec<HsmlNode>>,
    /// Source location spanning the `@for` head through any `@empty` block.
    pub location: Location,
}

// PartialEq excludes location so that tests comparing parsed ASTs
// don't need to specify exact location values.
impl PartialEq for ForNode {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression
            && self.body == other.body
            && self.empty_branch == other.empty_branch
    }
}

impl ForNode {
    /// Create a ForNode without a meaningful source location (for tests).
    #[doc(hidden)]
    pub fn new_without_location(
        expression: impl Into<String>,
        body: Vec<HsmlNode>,
        empty_branch: Option<Vec<HsmlNode>>,
    ) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            expression: expression.into(),
            body,
            empty_branch,
            location: Location {
                start: zero,
                end: zero,
            },
        }
    }
}

/// A `@switch (…)` block. Its only children are `@case` / `@default` branches
/// (no fall-through, matching Angular).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchNode {
    pub expression: String,
    pub cases: Vec<CaseNode>,
    pub default: Option<DefaultBranch>,
    /// Source location spanning the whole `@switch` block.
    pub location: Location,
}

// PartialEq excludes location so that tests comparing parsed ASTs
// don't need to specify exact location values.
impl PartialEq for SwitchNode {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression
            && self.cases == other.cases
            && self.default == other.default
    }
}

impl SwitchNode {
    /// Create a SwitchNode without a meaningful source location (for tests).
    #[doc(hidden)]
    pub fn new_without_location(
        expression: impl Into<String>,
        cases: Vec<CaseNode>,
        default: Option<DefaultBranch>,
    ) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            expression: expression.into(),
            cases,
            default,
            location: Location {
                start: zero,
                end: zero,
            },
        }
    }
}

/// One `@case` branch. `values` holds one entry normally, or several when
/// consecutive bare `@case` blocks stack onto a shared `body` (e.g.
/// `@case (b)` / `@case (c)` → `values = ["b", "c"]`). An empty `body`
/// represents an explicit `@case (x) {}`.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseNode {
    pub values: Vec<String>,
    pub body: Vec<HsmlNode>,
}

/// The `@default` branch of a `@switch`: either a normal block or an
/// exhaustiveness assertion (`@default never;` / `@default never(expr);`).
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DefaultBranch {
    Block(Vec<HsmlNode>),
    /// `@default never;` (None) or `@default never(expr);` (Some(expr)).
    Never(Option<String>),
}

/// A `@defer` deferrable view with optional `@placeholder` / `@loading` /
/// `@error` sub-blocks.
///
/// `triggers` is the raw text inside `@defer (...)` (e.g. "on viewport; prefetch
/// on idle"), captured verbatim, or `None` when `@defer` has no trigger list.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeferNode {
    pub triggers: Option<String>,
    pub body: Vec<HsmlNode>,
    pub placeholder: Option<DeferBlock>,
    pub loading: Option<DeferBlock>,
    pub error: Option<Vec<HsmlNode>>,
    /// Source location spanning the whole `@defer` block and its sub-blocks.
    pub location: Location,
}

// PartialEq excludes location so that tests comparing parsed ASTs
// don't need to specify exact location values.
impl PartialEq for DeferNode {
    fn eq(&self, other: &Self) -> bool {
        self.triggers == other.triggers
            && self.body == other.body
            && self.placeholder == other.placeholder
            && self.loading == other.loading
            && self.error == other.error
    }
}

impl DeferNode {
    /// Create a DeferNode without a meaningful source location (for tests).
    #[doc(hidden)]
    pub fn new_without_location(
        triggers: Option<String>,
        body: Vec<HsmlNode>,
        placeholder: Option<DeferBlock>,
        loading: Option<DeferBlock>,
        error: Option<Vec<HsmlNode>>,
    ) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            triggers,
            body,
            placeholder,
            loading,
            error,
            location: Location {
                start: zero,
                end: zero,
            },
        }
    }
}

/// A `@placeholder` or `@loading` sub-block of a `@defer`: an optional
/// parenthesized parameter list (e.g. `(minimum 500ms)`) plus a body.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeferBlock {
    pub params: Option<String>,
    pub body: Vec<HsmlNode>,
}

/// A `@boundary` error boundary (Angular 22) with an optional `@catch` block.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundaryNode {
    pub body: Vec<HsmlNode>,
    pub catch: Option<CatchBlock>,
    /// Source location spanning the `@boundary` block and its `@catch`.
    pub location: Location,
}

// PartialEq excludes location so that tests comparing parsed ASTs
// don't need to specify exact location values.
impl PartialEq for BoundaryNode {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body && self.catch == other.catch
    }
}

impl BoundaryNode {
    /// Create a BoundaryNode without a meaningful source location (for tests).
    #[doc(hidden)]
    pub fn new_without_location(body: Vec<HsmlNode>, catch: Option<CatchBlock>) -> Self {
        let zero = Position { line: 0, column: 0 };
        Self {
            body,
            catch,
            location: Location {
                start: zero,
                end: zero,
            },
        }
    }
}

/// The `@catch` block of a `@boundary`: an optional `(error)` binding plus a body.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatchBlock {
    pub binding: Option<String>,
    pub body: Vec<HsmlNode>,
}

fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Parse any Angular `@`-block, dispatching on the keyword following `@`.
pub fn angular_node<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, AngularNode> {
    let fragment = *input.fragment();

    if fragment.starts_with("@let") {
        let (rest, node) = let_node(input)?;
        return Ok((rest, AngularNode::Let(node)));
    }

    if fragment.starts_with("@if") {
        let (rest, node) = if_node(input, context)?;
        return Ok((rest, AngularNode::If(node)));
    }

    if fragment.starts_with("@for") {
        let (rest, node) = for_node(input, context)?;
        return Ok((rest, AngularNode::For(node)));
    }

    if fragment.starts_with("@switch") {
        let (rest, node) = switch_node(input, context)?;
        return Ok((rest, AngularNode::Switch(node)));
    }

    if fragment.starts_with("@defer") {
        let (rest, node) = defer_node(input, context)?;
        return Ok((rest, AngularNode::Defer(node)));
    }

    if fragment.starts_with("@boundary") {
        let (rest, node) = boundary_node(input, context)?;
        return Ok((rest, AngularNode::Boundary(node)));
    }

    Err(HsmlError::fail_msg(
        input,
        "Unknown Angular block (expected one of: @let, @if, @for, @switch, @defer, @boundary)",
    ))
}

/// Parse an `@let name = expression;` declaration.
pub fn let_node(input: Span<'_>) -> HsmlResult<'_, LetNode> {
    let start = input;

    let (input, _) = tag("@let")(input)?;
    // `@let` must be followed by at least one space/tab (not a newline).
    let (input, _) = space1(input)?;

    // Variable name — a JS identifier (kept lenient; HSML never evaluates it).
    let (input, name) = take_while1(is_identifier_char)(input)?;

    let (input, _) = space0(input)?;
    let (input, _) = char('=')(input)?;
    let (input, _) = space0(input)?;

    // Expression: captured verbatim up to the first top-level `;` (may be multi-line).
    let expr_span = input;
    let fragment = *expr_span.fragment();

    let Some(end) = find_let_expression_end(fragment) else {
        return Err(HsmlError::fail_msg(
            expr_span,
            "Unterminated @let declaration: expected ';'",
        ));
    };

    let expression = fragment[..end].trim_end().to_string();
    let rest = advance(expr_span, end + 1); // consume the terminating ';'

    Ok((
        rest,
        LetNode {
            name: name.fragment().to_string(),
            expression,
            location: Location::from_spans(&start, &rest),
        },
    ))
}

enum Continuation {
    ElseIf,
    Else,
}

/// Parse an `@if (…)` block, including any `@else if (…)` / `@else` branches.
pub fn if_node<'a>(input: Span<'a>, context: &mut HsmlProcessContext) -> HsmlResult<'a, IfNode> {
    let start = input;

    let (input, _) = tag("@if")(input)?;
    let (input, condition) = paren_condition(input)?;
    let (mut input, then_branch) = parse_block_body(input, context)?;

    let mut else_if_branches: Vec<ElseIfBranch> = vec![];
    let mut else_branch: Option<Vec<HsmlNode>> = None;

    // Continuations (`@else if` / `@else`) sit at the same indentation as `@if`.
    while let Some((cont_input, kind)) = try_continuation(input, &context.indent_string) {
        match kind {
            Continuation::ElseIf => {
                let (after, _) = tag("@else")(cont_input)?;
                let (after, _) = space1(after)?;
                let (after, _) = tag("if")(after)?;
                let (after, condition) = paren_condition(after)?;
                let (after, body) = parse_block_body(after, context)?;
                else_if_branches.push(ElseIfBranch { condition, body });
                input = after;
            }
            Continuation::Else => {
                let (after, _) = tag("@else")(cont_input)?;
                let (after, body) = parse_block_body(after, context)?;
                else_branch = Some(body);
                input = after;
                break; // `@else` terminates the chain
            }
        }
    }

    Ok((
        input,
        IfNode {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
            location: Location::from_spans(&start, &input),
        },
    ))
}

/// Parse a `@for (… ; track …)` loop, including an optional `@empty` block.
pub fn for_node<'a>(input: Span<'a>, context: &mut HsmlProcessContext) -> HsmlResult<'a, ForNode> {
    let start = input;

    let (input, _) = tag("@for")(input)?;
    let (input, expression) = paren_condition(input)?;
    let (mut input, body) = parse_block_body(input, context)?;

    // The `@empty` block sits at the same indentation as `@for`.
    let mut empty_branch = None;
    if let Some(at_empty) = peek_block_keyword(input, &context.indent_string, "@empty") {
        let (after, _) = tag("@empty")(at_empty)?;
        let (after, empty_body) = parse_block_body(after, context)?;
        empty_branch = Some(empty_body);
        input = after;
    }

    Ok((
        input,
        ForNode {
            expression,
            body,
            empty_branch,
            location: Location::from_spans(&start, &input),
        },
    ))
}

/// Parse a `@switch (…)` block and its `@case` / `@default` branches.
pub fn switch_node<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, SwitchNode> {
    let start = input;

    let (input, _) = tag("@switch")(input)?;
    let (mut input, expression) = paren_condition(input)?;

    let switch_indent = context.indent_string.clone();
    let mut cases: Vec<CaseNode> = vec![];
    let mut default: Option<DefaultBranch> = None;
    // Bare `@case` values awaiting the body of the case they stack onto.
    let mut pending_values: Vec<String> = vec![];

    while let Some((at_content, line_indent)) = peek_deeper_line(input, &switch_indent) {
        let saved_indent = context.indent_string.clone();
        let saved_level = context.nested_tag_level;
        context.indent_string = line_indent;
        context.nested_tag_level += 1;

        let parsed = parse_switch_child(at_content, context);

        context.indent_string = saved_indent;
        context.nested_tag_level = saved_level;

        let (after, child) = parsed?;
        match child {
            SwitchChild::Case { value, body } => {
                pending_values.push(value);
                if let Some(body) = body {
                    cases.push(CaseNode {
                        values: std::mem::take(&mut pending_values),
                        body,
                    });
                }
                input = after;
            }
            SwitchChild::Default(branch) => {
                if !pending_values.is_empty() {
                    return Err(HsmlError::fail_msg(
                        after,
                        "A bare @case must be followed by a @case with a body",
                    ));
                }
                if default.is_some() {
                    return Err(HsmlError::fail_msg(
                        after,
                        "@switch may only have one @default",
                    ));
                }
                default = Some(branch);
                input = after;
            }
        }
    }

    if !pending_values.is_empty() {
        return Err(HsmlError::fail_msg(
            input,
            "A bare @case must be followed by a @case with a body",
        ));
    }

    Ok((
        input,
        SwitchNode {
            expression,
            cases,
            default,
            location: Location::from_spans(&start, &input),
        },
    ))
}

enum SwitchChild {
    /// `body` is `None` for a bare (stacking) `@case`, `Some` otherwise
    /// (including `Some(vec![])` for an explicit `{}` empty case).
    Case {
        value: String,
        body: Option<Vec<HsmlNode>>,
    },
    Default(DefaultBranch),
}

/// Parse one `@case` / `@default` line inside a `@switch` body.
fn parse_switch_child<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, SwitchChild> {
    let fragment = *input.fragment();

    if fragment.starts_with("@case") {
        let (input, _) = tag("@case")(input)?;
        let (input, value) = paren_condition(input)?;
        let (input, body) = parse_case_body(input, context)?;
        return Ok((input, SwitchChild::Case { value, body }));
    }

    if fragment.starts_with("@default") {
        let (input, branch) = parse_default(input, context)?;
        return Ok((input, SwitchChild::Default(branch)));
    }

    Err(HsmlError::fail_msg(
        input,
        "@switch may only contain @case and @default blocks",
    ))
}

/// Parse a `@case` body: `Some(children)` for an indented block, `Some(vec![])`
/// for an explicit `{}`, or `None` for a bare case that stacks onto the next one.
fn parse_case_body<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, Option<Vec<HsmlNode>>> {
    let (input, _) = space0(input)?;

    if input.fragment().starts_with("{}") {
        return Ok((advance(input, 2), Some(vec![])));
    }

    if !(input.starts_with('\n') || input.starts_with("\r\n")) {
        return Err(HsmlError::fail_msg(
            input,
            "Expected a @case body, `{}`, or a stacked @case",
        ));
    }

    let (rest, children) = parse_children(input, context)?;
    if children.is_empty() {
        Ok((rest, None))
    } else {
        Ok((rest, Some(children)))
    }
}

/// Parse a `@default` branch: a normal block (`@default` + body / `{}`) or an
/// exhaustiveness assertion (`@default never;` / `@default never(expr);`).
fn parse_default<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, DefaultBranch> {
    let (input, _) = tag("@default")(input)?;
    let (input, _) = space0(input)?;

    let fragment = *input.fragment();
    if let Some(after_never) = fragment.strip_prefix("never")
        && (after_never.is_empty() || after_never.starts_with([' ', '\t', ';', '(']))
    {
        let input = advance(input, "never".len());
        let (input, _) = space0(input)?;

        let frag = *input.fragment();
        let (input, expression) = if frag.starts_with('(') {
            let Some(len) = balanced_parens_len(frag) else {
                return Err(HsmlError::fail_msg(
                    input,
                    "Unbalanced parentheses in @default never(...)",
                ));
            };
            (
                advance(input, len),
                Some(frag[1..len - 1].trim().to_string()),
            )
        } else {
            (input, None)
        };

        let (input, _) = space0(input)?;
        let (input, _) = char(';')(input)?;
        return Ok((input, DefaultBranch::Never(expression)));
    }

    let (input, body) = parse_block_body(input, context)?;
    Ok((input, DefaultBranch::Block(body)))
}

/// If the next line after `input` is indented deeper than `parent_indent`,
/// return the span at its content plus that line's indentation string;
/// otherwise `None` with `input` left untouched.
fn peek_deeper_line<'a>(input: Span<'a>, parent_indent: &str) -> Option<(Span<'a>, String)> {
    if !(input.starts_with('\n') || input.starts_with("\r\n")) {
        return None;
    }

    let (cursor, _) = advance_to_next_line(input).ok()?;
    let fragment = *cursor.fragment();
    if fragment.trim().is_empty() {
        return None;
    }

    let content = fragment.trim_start_matches([' ', '\t']);
    let indent_len = fragment.len() - content.len();
    let line_indent = &fragment[..indent_len];

    if line_indent.starts_with(parent_indent) && line_indent.len() > parent_indent.len() {
        Some((advance(cursor, indent_len), line_indent.to_string()))
    } else {
        None
    }
}

/// Parse a `@defer` block and its optional `@placeholder` / `@loading` /
/// `@error` sub-blocks.
pub fn defer_node<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, DeferNode> {
    let start = input;

    let (input, _) = tag("@defer")(input)?;
    let (input, triggers) = optional_paren_head(input)?;
    let (mut input, body) = parse_block_body(input, context)?;

    let block_indent = context.indent_string.clone();
    let mut placeholder = None;
    let mut loading = None;
    let mut error = None;

    // Sub-blocks sit at the same indentation as `@defer`, in any order.
    loop {
        if let Some(at) = peek_block_keyword(input, &block_indent, "@placeholder") {
            if placeholder.is_some() {
                return Err(HsmlError::fail_msg(
                    at,
                    "@defer may only have one @placeholder",
                ));
            }
            let (after, block) = parse_defer_sub_block(at, "@placeholder", context)?;
            placeholder = Some(block);
            input = after;
        } else if let Some(at) = peek_block_keyword(input, &block_indent, "@loading") {
            if loading.is_some() {
                return Err(HsmlError::fail_msg(at, "@defer may only have one @loading"));
            }
            let (after, block) = parse_defer_sub_block(at, "@loading", context)?;
            loading = Some(block);
            input = after;
        } else if let Some(at) = peek_block_keyword(input, &block_indent, "@error") {
            if error.is_some() {
                return Err(HsmlError::fail_msg(at, "@defer may only have one @error"));
            }
            let (after, _) = tag("@error")(at)?;
            let (after, body) = parse_block_body(after, context)?;
            error = Some(body);
            input = after;
        } else {
            break;
        }
    }

    Ok((
        input,
        DeferNode {
            triggers,
            body,
            placeholder,
            loading,
            error,
            location: Location::from_spans(&start, &input),
        },
    ))
}

/// Parse a `@placeholder` / `@loading` sub-block (optional parenthesized params
/// followed by a body) positioned at its keyword.
fn parse_defer_sub_block<'a>(
    at: Span<'a>,
    keyword: &str,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, DeferBlock> {
    let (input, _) = tag(keyword)(at)?;
    let (input, params) = optional_paren_head(input)?;
    let (input, body) = parse_block_body(input, context)?;
    Ok((input, DeferBlock { params, body }))
}

/// Parse an optional ` (…)` head, returning its verbatim inner text if present.
fn optional_paren_head(input: Span<'_>) -> HsmlResult<'_, Option<String>> {
    let (input, _) = space0(input)?;

    let fragment = *input.fragment();
    if !fragment.starts_with('(') {
        return Ok((input, None));
    }

    let Some(len) = balanced_parens_len(fragment) else {
        return Err(HsmlError::fail_msg(
            input,
            "Unbalanced parentheses in block head",
        ));
    };

    Ok((
        advance(input, len),
        Some(fragment[1..len - 1].trim().to_string()),
    ))
}

/// Parse a `@boundary` error boundary with an optional `@catch (error)` block.
pub fn boundary_node<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, BoundaryNode> {
    let start = input;

    let (input, _) = tag("@boundary")(input)?;
    let (mut input, body) = parse_block_body(input, context)?;

    let block_indent = context.indent_string.clone();
    let mut catch = None;
    if let Some(at) = peek_block_keyword(input, &block_indent, "@catch") {
        let (after, _) = tag("@catch")(at)?;
        let (after, binding) = optional_paren_head(after)?;
        let (after, catch_body) = parse_block_body(after, context)?;
        catch = Some(CatchBlock {
            binding,
            body: catch_body,
        });
        input = after;
    }

    Ok((
        input,
        BoundaryNode {
            body,
            catch,
            location: Location::from_spans(&start, &input),
        },
    ))
}

/// Parse ` (condition)` after a block keyword, returning the verbatim inner text
/// (without the surrounding parentheses). The head may span multiple lines.
fn paren_condition(input: Span<'_>) -> HsmlResult<'_, String> {
    let (input, _) = space0(input)?;

    let fragment = *input.fragment();
    let Some(len) = balanced_parens_len(fragment) else {
        return Err(HsmlError::fail_msg(
            input,
            "Expected a parenthesized condition",
        ));
    };

    let condition = fragment[1..len - 1].trim().to_string();
    Ok((advance(input, len), condition))
}

/// Parse a block body following a head: either an explicit empty `{}` on the
/// same line, or an indented child block on the following lines. A bare head
/// with neither is rejected (use `{}` to declare an intentionally empty block).
fn parse_block_body<'a>(
    input: Span<'a>,
    context: &mut HsmlProcessContext,
) -> HsmlResult<'a, Vec<HsmlNode>> {
    let (input, _) = space0(input)?;

    if input.fragment().starts_with("{}") {
        return Ok((advance(input, 2), vec![]));
    }

    if !(input.starts_with('\n') || input.starts_with("\r\n")) {
        return Err(HsmlError::fail_msg(
            input,
            "Expected a block body (indented children) or `{}` for an empty block",
        ));
    }

    let (rest, children) = parse_children(input, context)?;

    if children.is_empty() {
        return Err(HsmlError::fail_msg(
            input,
            "Empty block body — use `{}` to declare an intentionally empty block",
        ));
    }

    Ok((rest, children))
}

/// Peek past `input` (which should sit at a newline ending a block body) for a
/// continuation `keyword` (e.g. `@else`, `@empty`) at exactly `indent`. On a
/// match the returned span is positioned at the keyword (the newline and
/// indentation are consumed); otherwise `None` and `input` is left untouched.
fn peek_block_keyword<'a>(input: Span<'a>, indent: &str, keyword: &str) -> Option<Span<'a>> {
    if !(input.starts_with('\n') || input.starts_with("\r\n")) {
        return None;
    }

    let (cursor, _) = advance_to_next_line(input).ok()?;
    let fragment = *cursor.fragment();
    if fragment.trim().is_empty() {
        return None;
    }

    let content = fragment.trim_start_matches([' ', '\t']);
    let indent_len = fragment.len() - content.len();
    if &fragment[..indent_len] != indent {
        return None;
    }

    let after = content.strip_prefix(keyword)?;
    // Require a word boundary so e.g. `@elsewhere` / `@empties` aren't matched.
    // `(` is allowed so no-space heads like `@catch(error)` are still recognized.
    if !(after.is_empty() || after.starts_with([' ', '\t', '\n', '\r', '{', '('])) {
        return None;
    }

    Some(advance(cursor, indent_len))
}

/// Detect an `@else` / `@else if` continuation at `indent` (see [`peek_block_keyword`]).
fn try_continuation<'a>(input: Span<'a>, indent: &str) -> Option<(Span<'a>, Continuation)> {
    let at_keyword = peek_block_keyword(input, indent, "@else")?;

    let after = (*at_keyword.fragment()).strip_prefix("@else").unwrap_or("");
    let is_else_if = after
        .trim_start_matches([' ', '\t'])
        .strip_prefix("if")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t', '(']));

    let kind = if is_else_if {
        Continuation::ElseIf
    } else {
        Continuation::Else
    };

    Some((at_keyword, kind))
}
