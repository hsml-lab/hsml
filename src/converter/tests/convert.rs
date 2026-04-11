use crate::converter::convert;

fn conv(html: &str) -> String {
    convert(html).unwrap()
}

// --- Basic tags ---

#[test]
fn it_should_convert_simple_tag() {
    assert_eq!(conv("<p>Hello</p>"), "p Hello\n");
}

#[test]
fn it_should_convert_empty_tag() {
    assert_eq!(conv("<div></div>"), "div\n");
}

#[test]
fn it_should_convert_nested_tags() {
    assert_eq!(conv("<div><p>Hello</p></div>"), "div\n  p Hello\n");
}

#[test]
fn it_should_convert_deeply_nested_tags() {
    assert_eq!(
        conv("<div><section><p>Hello</p></section></div>"),
        "div\n  section\n    p Hello\n"
    );
}

// --- Implicit div ---

#[test]
fn it_should_convert_div_with_class_to_implicit_div() {
    assert_eq!(conv("<div class=\"card\">Hello</div>"), ".card Hello\n");
}

#[test]
fn it_should_convert_div_with_id_to_implicit_div() {
    assert_eq!(conv("<div id=\"app\"></div>"), "#app\n");
}

#[test]
fn it_should_convert_div_with_id_and_classes() {
    assert_eq!(
        conv("<div id=\"app\" class=\"container main\"></div>"),
        "#app.container.main\n"
    );
}

#[test]
fn it_should_not_use_implicit_div_for_other_tags() {
    assert_eq!(
        conv("<span class=\"highlight\">text</span>"),
        "span.highlight text\n"
    );
}

// --- Attributes ---

#[test]
fn it_should_convert_attributes() {
    assert_eq!(
        conv("<a href=\"https://github.com\" target=\"_blank\">GitHub</a>"),
        "a(href=\"https://github.com\", target=\"_blank\") GitHub\n"
    );
}

#[test]
fn it_should_convert_boolean_attributes() {
    assert_eq!(
        conv("<button disabled>Click</button>"),
        "button(disabled) Click\n"
    );
}

#[test]
fn it_should_extract_id_and_class_from_attributes() {
    assert_eq!(
        conv("<img id=\"photo\" class=\"rounded\" src=\"photo.jpg\">"),
        "img#photo.rounded(src=\"photo.jpg\")\n"
    );
}

// --- IDs and classes with special characters ---

#[test]
fn it_should_escape_id_with_hash() {
    // id="foo#bar" can't use #foo#bar syntax — ambiguous
    assert_eq!(
        conv(r#"<div id="foo#bar"></div>"#),
        r#"div(id="foo#bar")
"#
    );
}

#[test]
fn it_should_escape_class_with_dot() {
    // class="btn.primary" can't use .btn.primary syntax — ambiguous
    assert_eq!(
        conv(r#"<div class="btn.primary"></div>"#),
        r#"div(class="btn.primary")
"#
    );
}

#[test]
fn it_should_fallback_entire_class_when_one_contains_dot() {
    // Safe classes use shorthand, unsafe ones fall back to attribute syntax
    assert_eq!(
        conv(r#"<div class="container btn.primary active"></div>"#),
        r#".container.active(class="btn.primary")
"#
    );
}

#[test]
fn it_should_use_shorthand_for_tailwind_arbitrary_color() {
    // bg-[#1da1f2] has # inside brackets — safe for shorthand
    assert_eq!(
        conv(r#"<div class="bg-[#1da1f2] text-white"></div>"#),
        ".bg-[#1da1f2].text-white\n"
    );
}

#[test]
fn it_should_handle_normal_id_and_class() {
    // Normal id/class without special chars — use shorthand
    assert_eq!(
        conv(r#"<div id="app" class="container main"></div>"#),
        "#app.container.main\n"
    );
}

// --- PascalCase tag preservation ---

#[test]
fn it_should_preserve_pascal_case_vue_components() {
    assert_eq!(
        conv(r#"<PwaInstallPrompt class="xl:hidden"></PwaInstallPrompt>"#),
        "PwaInstallPrompt.xl:hidden\n"
    );
}

#[test]
fn it_should_preserve_multiple_pascal_case_components() {
    assert_eq!(
        conv(
            r#"<div><NavUser v-if="show"></NavUser><NavUserSkeleton v-else></NavUserSkeleton></div>"#
        ),
        r#"div
  NavUser(v-if="show")
  NavUserSkeleton(v-else)
"#
    );
}

#[test]
fn it_should_preserve_pascal_case_in_mixed_content() {
    assert_eq!(
        conv(r#"<p>Hello <MyBadge>World</MyBadge> more</p>"#),
        "p Hello <MyBadge>World</MyBadge> more\n"
    );
}

// --- Kebab-case custom elements ---

#[test]
fn it_should_preserve_kebab_case_custom_elements() {
    assert_eq!(
        conv(r#"<my-component class="active"></my-component>"#),
        "my-component.active\n"
    );
}

#[test]
fn it_should_preserve_kebab_case_web_components() {
    assert_eq!(
        conv(r#"<pwa-install-prompt class="xl:hidden"></pwa-install-prompt>"#),
        "pwa-install-prompt.xl:hidden\n"
    );
}

#[test]
fn it_should_not_confuse_kebab_case_with_pascal_case() {
    // Both forms in the same document — each preserved as written
    assert_eq!(
        conv(
            r#"<div><PwaBadge class="lg:hidden"></PwaBadge><pwa-badge class="xl:hidden"></pwa-badge></div>"#
        ),
        r#"div
  PwaBadge.lg:hidden
  pwa-badge.xl:hidden
"#
    );
}

// --- Vue/Angular syntax ---

#[test]
fn it_should_preserve_vue_attributes() {
    assert_eq!(
        conv("<button @click=\"handleClick\" :class=\"dynamicClass\" v-if=\"show\">Click</button>"),
        "button(@click=\"handleClick\", :class=\"dynamicClass\", v-if=\"show\") Click\n"
    );
}

#[test]
fn it_should_convert_template_and_slot() {
    assert_eq!(
        conv("<template #default><p>Content</p></template>"),
        "template(#default)\n  p Content\n"
    );
}

// --- Void elements ---

#[test]
fn it_should_convert_void_elements() {
    assert_eq!(conv("<br>"), "br\n");
    assert_eq!(conv("<hr>"), "hr\n");
    assert_eq!(
        conv("<img src=\"photo.jpg\" alt=\"Photo\">"),
        "img(src=\"photo.jpg\", alt=\"Photo\")\n"
    );
    assert_eq!(
        conv("<input type=\"text\" placeholder=\"Name\">"),
        "input(type=\"text\", placeholder=\"Name\")\n"
    );
}

// --- Comments ---

#[test]
fn it_should_convert_comments() {
    assert_eq!(conv("<!-- Hello -->"), "//! Hello\n");
}

#[test]
fn it_should_convert_multiline_comment() {
    assert_eq!(
        conv("<!-- line one\nline two -->"),
        "//! line one\n//! line two\n"
    );
}

// --- DOCTYPE ---

#[test]
fn it_should_convert_doctype() {
    assert_eq!(
        conv("<!DOCTYPE html><html><head></head><body><p>Hello</p></body></html>"),
        "doctype html\nhtml\n  head\n  body\n    p Hello\n"
    );
}

// --- Mixed content ---

#[test]
fn it_should_keep_mixed_content_as_raw_html() {
    assert_eq!(
        conv("<p>Hello <strong>World</strong> more</p>"),
        "p Hello <strong>World</strong> more\n"
    );
}

#[test]
fn it_should_keep_complex_mixed_content_as_raw_html() {
    assert_eq!(
        conv("<p>Visit <a href=\"https://example.com\">example</a> now</p>"),
        "p Visit <a href=\"https://example.com\">example</a> now\n"
    );
}

#[test]
fn it_should_handle_multiline_mixed_content_as_text_block() {
    assert_eq!(
        conv("<p>Hello\n<strong>World</strong>\nmore</p>"),
        "p.\n  Hello\n  <strong>World</strong>\n  more\n"
    );
}

#[test]
fn it_should_handle_mixed_content_with_void_element() {
    assert_eq!(conv("<p>Hello<br>World</p>"), "p Hello<br />World\n");
}

#[test]
fn it_should_handle_mixed_content_with_comment() {
    // Comments interleaved with text are treated as mixed content
    assert_eq!(
        conv("<p>Hello<!-- separator -->World</p>"),
        "p Hello<!-- separator -->World\n"
    );
}

#[test]
fn it_should_handle_multiline_text_as_text_block() {
    assert_eq!(
        conv("<p>Line one\nLine two\nLine three</p>"),
        "p.\n  Line one\n  Line two\n  Line three\n"
    );
}

// --- HTML entities in mixed content ---

#[test]
fn it_should_escape_entities_in_mixed_content_text() {
    // html5ever decodes &amp; to & in the DOM — serialize_inner_html must re-encode
    assert_eq!(
        conv("<p>A &amp; B <strong>bold</strong></p>"),
        "p A &amp; B <strong>bold</strong>\n"
    );
}

#[test]
fn it_should_escape_lt_gt_in_mixed_content_text() {
    assert_eq!(
        conv("<p>Use &lt;div&gt; for <em>containers</em></p>"),
        "p Use &lt;div&gt; for <em>containers</em>\n"
    );
}

#[test]
fn it_should_escape_quotes_in_mixed_content_attributes() {
    assert_eq!(
        conv("<p>Click <a href=\"/search?q=a&amp;b\">here</a> now</p>"),
        "p Click <a href=\"/search?q=a&amp;b\">here</a> now\n"
    );
}

// --- HTML entities in HSML attribute output ---

#[test]
fn it_should_escape_quotes_in_hsml_attribute_values() {
    // An attribute value containing a double quote would break key="value" syntax
    assert_eq!(
        conv(r#"<div title="say &quot;hello&quot;"></div>"#),
        r#"div(title="say &quot;hello&quot;")
"#
    );
}

#[test]
fn it_should_reencode_ampersands_in_regular_hsml_attributes() {
    // html5ever decodes &amp; to & — regular HTML attributes must re-encode
    // so the compiled HTML output has valid &amp;
    assert_eq!(
        conv(r#"<a href="/search?a=1&amp;b=2">link</a>"#),
        r#"a(href="/search?a=1&amp;b=2") link
"#
    );
}

#[test]
fn it_should_keep_ampersands_in_vue_directive_values() {
    // Vue directive values contain JS expressions where && is valid
    assert_eq!(
        conv(r#"<div :class="a && b"></div>"#),
        r#"div(:class="a && b")
"#
    );
}

#[test]
fn it_should_keep_ampersands_in_angular_directive_values() {
    assert_eq!(
        conv(r#"<div [class]="a && b"></div>"#),
        r#"div([class]="a && b")
"#
    );
}

// --- Whitespace-sensitive tags ---

#[test]
fn it_should_preserve_whitespace_in_pre_tag() {
    // Pre tag content preserves original whitespace via text block syntax.
    // Text block indent is (depth+1)*2 = 2 spaces, original content on top.
    assert_eq!(
        conv("<pre>  line 1\n  line 2\n    indented</pre>"),
        "pre.\n    line 1\n    line 2\n      indented\n"
    );
}

#[test]
fn it_should_preserve_whitespace_in_pre_tag_with_class() {
    assert_eq!(
        conv(
            "<pre class=\"hljs language-rust\">fn main() {\n    println!(\"Hello, world!\");\n}</pre>"
        ),
        "pre.hljs.language-rust.\n  fn main() {\n      println!(\"Hello, world!\");\n  }\n"
    );
}

#[test]
fn it_should_preserve_whitespace_in_textarea() {
    assert_eq!(
        conv("<textarea>  some\n  text</textarea>"),
        "textarea.\n    some\n    text\n"
    );
}

#[test]
#[ignore] // TODO: round-trip for pre tags loses leading whitespace — needs text block compiler fix
fn it_should_roundtrip_pre_tag_content() {
    let html = "<pre>  line 1\n  line 2</pre>";
    let hsml = conv(html);

    let compiled = crate::compile_content_core(&hsml).unwrap();
    assert_eq!(compiled, html, "round-trip should preserve pre content");
}

// --- Pre with nested markup ---

#[test]
fn it_should_preserve_nested_markup_in_pre() {
    assert_eq!(
        conv("<pre><code>hello</code></pre>"),
        "pre.\n  <code>hello</code>\n"
    );
}

#[test]
fn it_should_preserve_nested_markup_with_whitespace_in_pre() {
    assert_eq!(
        conv("<pre><code>  fn main() {\n    println!(\"hi\");\n  }</code></pre>"),
        "pre.\n  <code>  fn main() {\n      println!(\"hi\");\n    }</code>\n"
    );
}

// --- Script/style raw text ---

#[test]
fn it_should_not_escape_script_content_in_mixed_serialization() {
    assert_eq!(
        conv("<script>if (a < b && c > d) {}</script>"),
        "script.\n  if (a < b && c > d) {}\n"
    );
}

#[test]
fn it_should_not_escape_style_content() {
    assert_eq!(
        conv("<style>.foo > .bar { color: red; }</style>"),
        "style.\n  .foo > .bar { color: red; }\n"
    );
}

#[test]
fn it_should_not_escape_script_in_mixed_content() {
    assert_eq!(
        conv(r#"<div>text<script>if (a < b) {}</script>more</div>"#),
        r#"div text<script>if (a < b) {}</script>more
"#
    );
}

// --- Self-closing non-void elements ---

#[test]
fn it_should_handle_self_closing_non_void_element() {
    // <div /> is not valid self-closing HTML (div is not a void element),
    // but frameworks like Vue use it commonly. html5ever treats <div /> as <div>
    // which swallows all subsequent siblings as children.
    assert_eq!(
        conv(r#"<div class="parent"><div class="child" /><div class="sibling">Hello</div></div>"#),
        ".parent\n  .child\n  .sibling Hello\n"
    );
}

#[test]
fn it_should_handle_self_closing_span() {
    assert_eq!(
        conv(r#"<div><span class="icon" /><span>Text</span></div>"#),
        "div\n  span.icon\n  span Text\n"
    );
}

#[test]
fn it_should_keep_void_elements_self_closing() {
    // img is a void element — self-closing is fine and should still work
    assert_eq!(
        conv(r#"<div><img src="a.jpg" /><p>Text</p></div>"#),
        "div\n  img(src=\"a.jpg\")\n  p Text\n"
    );
}

#[test]
fn it_should_keep_br_self_closing() {
    assert_eq!(
        conv(r#"<div><br /><p>Text</p></div>"#),
        "div\n  br\n  p Text\n"
    );
}

#[test]
fn it_should_handle_self_closing_with_attributes() {
    assert_eq!(
        conv(r#"<div class="a" data-x="y" />"#),
        ".a(data-x=\"y\")\n"
    );
}

#[test]
fn it_should_handle_self_closing_pascal_case_component() {
    assert_eq!(
        conv(r#"<div><MyComponent class="active" /><p>After</p></div>"#),
        "div\n  MyComponent.active\n  p After\n"
    );
}

#[test]
fn it_should_handle_multiple_self_closing_siblings() {
    assert_eq!(
        conv(r#"<div><div class="a" /><div class="b" /><div class="c" /></div>"#),
        "div\n  .a\n  .b\n  .c\n"
    );
}

#[test]
fn it_should_handle_self_closing_in_comment_context() {
    // Self-closing divs inside comments should not be expanded
    assert_eq!(
        conv(r#"<!-- <div class="ignore" /> --><p>Hello</p>"#),
        "//! <div class=\"ignore\" />\np Hello\n"
    );
}

#[test]
fn it_should_handle_elk_skeleton_html() {
    let html = r#"<div>
  <div px2 pt2>
    <div rounded of-hidden aspect="3.19" class="flex skeleton-loading-bg" />
    <div px-4 pb-4 flex="~ col gap-2">
      <div flex sm:flex-row flex-col flex-gap-2>
        <div flex items-center justify-between>
          <div w-17 h-17 rounded-full border-4 border-bg-base z-2 mt--2 ms--1 of-hidden bg-base>
            <div class="flex skeleton-loading-bg" w-full h-full />
          </div>
          <div block sm:hidden class="skeleton-loading-bg" h-8 w-30 rounded-full />
        </div>
        <div sm:mt-2 flex="~ col 1 gap-2">
          <div flex class="skeleton-loading-bg" h-5 w-20 rounded />
          <div flex class="skeleton-loading-bg" h-4 w-40 rounded />
        </div>
      </div>
      <div flex class="skeleton-loading-bg" h-4 my3 w="3/5" rounded />
      <div flex justify-between items-center>
        <div flex class="skeleton-loading-bg" h-4 w="sm:1/2 full" rounded />
        <div sm:flex hidden class="skeleton-loading-bg" h-8 w-30 rounded-full />
      </div>
    </div>
  </div>
</div>"#;

    let result = conv(html);

    // Each self-closing div should be a sibling, not swallowing subsequent elements.
    // The skeleton-loading-bg divs that are self-closing should not nest their siblings.
    // Verify key structural properties:
    // 1. "px-4" div should be a sibling of the first skeleton-loading-bg, not a child
    assert!(
        !result.contains("      .flex.skeleton-loading-bg\n        div"),
        "self-closing div should not swallow siblings as children"
    );
    // 2. The result should have the right number of skeleton-loading-bg occurrences
    assert_eq!(
        result.matches("skeleton-loading-bg").count(),
        8,
        "all 8 skeleton-loading-bg elements should be present"
    );
}

// --- TailwindCSS ---

#[test]
fn it_should_handle_tailwind_classes() {
    assert_eq!(
        conv("<div class=\"bg-[#1da1f2] lg:flex md:p-0\">Hello</div>"),
        ".bg-[#1da1f2].lg:flex.md:p-0 Hello\n"
    );
}

// --- Multiple root elements ---

#[test]
fn it_should_handle_multiple_root_elements() {
    assert_eq!(conv("<p>One</p><p>Two</p>"), "p One\np Two\n");
}

// --- Whitespace ---

#[test]
fn it_should_ignore_insignificant_whitespace() {
    assert_eq!(conv("<div>\n  <p>Hello</p>\n</div>"), "div\n  p Hello\n");
}

// --- Explicit html/body without DOCTYPE ---

#[test]
fn it_should_preserve_explicit_html_body_without_doctype() {
    // <head> is synthesized by html5ever but not in the source — should be skipped
    assert_eq!(
        conv("<html><body><p>Hello</p></body></html>"),
        "html\n  body\n    p Hello\n"
    );
}

// --- Real-world examples ---

#[test]
fn it_should_convert_vue_card_component_html() {
    let html = r#"<h1 class="text-red">Vite CJS Faker Demo</h1>
<div class="card">
  <div class="card__image">
    <img :src="natureImageUrl" :alt="'Background image for ' + fullName" />
  </div>
  <div class="card__profile">
    <img :src="avatarUrl" :alt="'Avatar image of ' + fullName" />
  </div>
  <div class="card__body">{{ fullName }}</div>
</div>"#;

    assert_eq!(
        conv(html),
        r#"h1.text-red Vite CJS Faker Demo
.card
  .card__image
    img(:src="natureImageUrl", :alt="'Background image for ' + fullName")
  .card__profile
    img(:src="avatarUrl", :alt="'Avatar image of ' + fullName")
  .card__body {{ fullName }}
"#
    );
}

#[test]
fn it_should_convert_tailwind_figure_html() {
    let html = r#"<!-- test comment on root layer -->
<figure class="md:flex bg-slate-100 rounded-xl p-8 md:p-0 dark:bg-slate-800/10">
  <!-- test comment --><img class="w-24 h-24 md:w-48 md:h-auto md:rounded-none rounded-full mx-auto" src="/fancy-avatar.jpg" alt="" width="384" height="512" />
  <div class="pt-6 md:p-8 text-center md:text-left space-y-4">
    <blockquote v-if="showBlockquote">
      <p class="text-lg font-medium">"Tailwind CSS is the only framework that I've seen scale on large teams. It's easy to customize, adapts to any design, and the build size is tiny."</p>
    </blockquote>
    <figcaption class="font-medium">
      <div class="text-sky-500 dark:text-sky-400">Sarah Dayan</div>
      <div class="text-[#af05c9] dark:text-slate-500">Staff Engineer, Algolia</div>
    </figcaption>
  </div>
</figure>"#;

    assert_eq!(
        conv(html),
        r#"//! test comment on root layer
figure.md:flex.bg-slate-100.rounded-xl.p-8.md:p-0.dark:bg-slate-800/10
  //! test comment
  img.w-24.h-24.md:w-48.md:h-auto.md:rounded-none.rounded-full.mx-auto(src="/fancy-avatar.jpg", alt, width="384", height="512")
  .pt-6.md:p-8.text-center.md:text-left.space-y-4
    blockquote(v-if="showBlockquote")
      p.text-lg.font-medium "Tailwind CSS is the only framework that I've seen scale on large teams. It's easy to customize, adapts to any design, and the build size is tiny."
    figcaption.font-medium
      .text-sky-500.dark:text-sky-400 Sarah Dayan
      .text-[#af05c9].dark:text-slate-500 Staff Engineer, Algolia
"#
    );
}

#[test]
fn it_should_convert_complex_vue_html() {
    let html = r#"<div ref="container" :class="containerClass">
  <div
    class="sticky top-0 z-20"
    pt="[env(safe-area-inset-top,0)]"
    bg="[rgba(var(--rgb-bg-base),0.7)]"
    :class="{
      'backdrop-blur': !getPreferences(userSettings, 'optimizeForLowPerformanceDevice'),
    }"
  >
    <div
      class="min-h-53px px-2 py-1"
      flex="~ justify-between"
      :class="{ 'xl:hidden': $route.name !== 'tag' }"
      border="b base"
    >
      <div class="w-full" flex="~ items-center">
        <button
          class="btn-text flex items-center p-3 xl:hidden"
          v-if="backOnSmallScreen || showBackButton"
          :aria-label="$t('nav.back')"
          @click="$router.go(-1)"
        >
          <div class="text-lg rtl-flip" i-ri:arrow-left-line></div>
        </button>
        <div class="flex w-full"><slot name="title"></slot></div>
        <div class="sm:hidden h-7 w-1px"></div>
      </div>
      <div class="px-3" flex="~ items-center shrink-0 gap-x-2">
        <slot name="actions"></slot>
        <PwaBadge class="xl:hidden"></PwaBadge>
        <NavUser v-if="isHydrated"></NavUser>
        <NavUserSkeleton v-else></NavUserSkeleton>
      </div>
    </div>
    <slot name="header"><div hidden></div></slot>
  </div>
  <PwaInstallPrompt class="xl:hidden"></PwaInstallPrompt>
  <div
    class="m-auto"
    :class="isHydrated && wideLayout ? 'xl:w-full sm:max-w-600px' : 'sm:max-w-600px md:shrink-0'"
  >
    <div
      class="h-6"
      hidden
      :class="{ 'xl:block': $route.name !== 'tag' && !$slots.header }"
    ></div>
    <slot></slot>
  </div>
</div>
"#;

    assert_eq!(
        conv(html),
        r#"div(ref="container", :class="containerClass")
  .sticky.top-0.z-20(pt="[env(safe-area-inset-top,0)]", bg="[rgba(var(--rgb-bg-base),0.7)]", :class="{
      'backdrop-blur': !getPreferences(userSettings, 'optimizeForLowPerformanceDevice'),
    }")
    .min-h-53px.px-2.py-1(flex="~ justify-between", :class="{ 'xl:hidden': $route.name !== 'tag' }", border="b base")
      .w-full(flex="~ items-center")
        button.btn-text.flex.items-center.p-3.xl:hidden(v-if="backOnSmallScreen || showBackButton", :aria-label="$t('nav.back')", @click="$router.go(-1)")
          .text-lg.rtl-flip(i-ri:arrow-left-line)
        .flex.w-full
          slot(name="title")
        .sm:hidden.h-7.w-1px
      .px-3(flex="~ items-center shrink-0 gap-x-2")
        slot(name="actions")
        PwaBadge.xl:hidden
        NavUser(v-if="isHydrated")
        NavUserSkeleton(v-else)
    slot(name="header")
      div(hidden)
  PwaInstallPrompt.xl:hidden
  .m-auto(:class="isHydrated && wideLayout ? 'xl:w-full sm:max-w-600px' : 'sm:max-w-600px md:shrink-0'")
    .h-6(hidden, :class="{ 'xl:block': $route.name !== 'tag' && !$slots.header }")
    slot
"#
    );
}
