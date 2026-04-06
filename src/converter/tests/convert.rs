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
    // TODO @Shinigami92 2026-04-06: PascalCase tags are currently not supported by html5ever
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
        <pwa-badge class="xl:hidden"></pwa-badge>
        <nav-user v-if="isHydrated"></nav-user>
        <nav-user-skeleton v-else></nav-user-skeleton>
      </div>
    </div>
    <slot name="header"><div hidden></div></slot>
  </div>
  <pwa-install-prompt class="xl:hidden"></pwa-install-prompt>
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
        pwa-badge.xl:hidden
        nav-user(v-if="isHydrated")
        nav-user-skeleton(v-else)
    slot(name="header")
      div(hidden)
  pwa-install-prompt.xl:hidden
  .m-auto(:class="isHydrated && wideLayout ? 'xl:w-full sm:max-w-600px' : 'sm:max-w-600px md:shrink-0'")
    .h-6(hidden, :class="{ 'xl:block': $route.name !== 'tag' && !$slots.header }")
    slot
"#
    );
}
