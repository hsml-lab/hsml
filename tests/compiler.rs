use hsml::{
    check_content, compile_content_core, compile_content_diagnostics,
    compile_content_diagnostics_with_options,
    compiler::{HsmlCompileOptions, compile},
    format_content_core,
    parser::{
        HsmlNode, RootNode, Span, class::node::ClassNode, doctype::node::DoctypeNode,
        error::ErrorCode, id::node::IdNode, parse::parse, tag::node::TagNode, text::node::TextNode,
    },
};

#[test]
fn it_should_compile_empty_ast() {
    let ast = RootNode { nodes: vec![] };

    let html_content = compile(&ast, &HsmlCompileOptions::default()).unwrap();

    assert_eq!(html_content, "");
}

#[test]
fn it_should_compile_simple_tag() {
    let ast = RootNode {
        nodes: vec![HsmlNode::Tag(TagNode::without_location(
            "h1",
            vec![],
            None,
            None,
            Some(TextNode {
                text: String::from("Hello World"),
                is_block: false,
            }),
            None,
        ))],
    };

    let html_content = compile(&ast, &HsmlCompileOptions::default()).unwrap();

    assert_eq!(html_content, "<h1>Hello World</h1>");
}

#[test]
fn it_should_compile_content_with_id() {
    let ast = RootNode {
        nodes: vec![HsmlNode::Tag(TagNode::without_location(
            "h1",
            vec![IdNode::new_without_location("title")],
            None,
            None,
            Some(TextNode {
                text: String::from("Hello World"),
                is_block: false,
            }),
            None,
        ))],
    };

    let html_content = compile(&ast, &HsmlCompileOptions::default()).unwrap();

    assert_eq!(html_content, r#"<h1 id="title">Hello World</h1>"#);
}

#[test]
fn it_should_compile_parsed_content() {
    let input = r#"h1.text-red Vite CJS Faker Demo
.card
  .card__image
    img(:src="natureImageUrl" :alt="'Background image for ' + fullName")
  .card__profile
    img(:src="avatarUrl" :alt="'Avatar image of ' + fullName")
  .card__body {{ fullName }}
"#;

    let (rest, ast) = parse(Span::new(input)).unwrap();

    let html_content = compile(&ast, &HsmlCompileOptions::default()).unwrap();

    assert_eq!(
        html_content,
        r#"<h1 class="text-red">Vite CJS Faker Demo</h1><div class="card"><div class="card__image"><img :src="natureImageUrl" :alt="'Background image for ' + fullName" /></div><div class="card__profile"><img :src="avatarUrl" :alt="'Avatar image of ' + fullName" /></div><div class="card__body">{{ fullName }}</div></div>"#
    );
    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_compile_parsed_content_2() {
    let input = r#"//! test comment on root layer
figure.md:flex.bg-slate-100.rounded-xl.p-8.md:p-0.dark:bg-slate-800/10
  //! test comment
  img.w-24.h-24.md:w-48.md:h-auto.md:rounded-none.rounded-full.mx-auto(
    // supports attribute inline comments
    src="/fancy-avatar.jpg"
    alt=""
    width="384"
    height="512"
  )
  div.pt-6.md:p-8.text-center.md:text-left.space-y-4
    blockquote(v-if="showBlockquote")
      p.text-lg.font-medium.
        "Tailwind CSS is the only framework that I've seen scale
        on large teams. It's easy to customize, adapts to any design,
        and the build size is tiny."
    figcaption.font-medium
      .text-sky-500.dark:text-sky-400.
        Sarah Dayan
      .text-[#af05c9].dark:text-slate-500.
        Staff Engineer, Algolia
"#;

    let (rest, ast) = parse(Span::new(input)).unwrap();

    let html_content = compile(&ast, &HsmlCompileOptions::default()).unwrap();

    assert_eq!(
        html_content,
        r#"<!-- test comment on root layer --><figure class="md:flex bg-slate-100 rounded-xl p-8 md:p-0 dark:bg-slate-800/10"><!-- test comment --><img class="w-24 h-24 md:w-48 md:h-auto md:rounded-none rounded-full mx-auto" src="/fancy-avatar.jpg" alt="" width="384" height="512" /><div class="pt-6 md:p-8 text-center md:text-left space-y-4"><blockquote v-if="showBlockquote"><p class="text-lg font-medium">"Tailwind CSS is the only framework that I've seen scale
on large teams. It's easy to customize, adapts to any design,
and the build size is tiny."</p></blockquote><figcaption class="font-medium"><div class="text-sky-500 dark:text-sky-400">Sarah Dayan</div><div class="text-[#af05c9] dark:text-slate-500">Staff Engineer, Algolia</div></figcaption></div></figure>"#
    );
    assert_eq!(*rest.fragment(), "");
}

#[test]
fn it_should_compile_parsed_elk_status_content_component() {
    let input = r#".space-y-3(
  :class="{
    'pt2 pb0.5 px3.5 bg-dm rounded-4 me--1': isDM,
    'ms--3.5 mt--1 ms--1': isDM && context !== 'details',
  }"
)
  StatusBody(v-if="(!isFiltered && isSensitiveNonSpoiler) || hideAllMedia" :status="status" :newer="newer" :with-action="!isDetails" :class="isDetails ? 'text-xl' : ''")
  StatusSpoiler(:enabled="hasSpoilerOrSensitiveMedia || isFiltered" :filter="isFiltered" :sensitive-non-spoiler="isSensitiveNonSpoiler || hideAllMedia" :is-d-m="isDM")
    template(v-if="spoilerTextPresent" #spoiler)
      p {{ status.spoilerText }}
    template(v-else-if="filterPhrase" #spoiler)
      p {{ `${$t('status.filter_hidden_phrase')}: ${filterPhrase}` }}
    StatusBody(v-if="!(isSensitiveNonSpoiler || hideAllMedia)" :status="status" :newer="newer" :with-action="!isDetails" :class="isDetails ? 'text-xl' : ''")
    StatusTranslation(:status="status")
    StatusPoll(v-if="status.poll" :status="status")
    StatusMedia(
      v-if="status.mediaAttachments?.length"
      :status="status"
      :is-preview="isPreview"
    )
    StatusPreviewCard(
      v-if="status.card"
      :card="status.card"
      :small-picture-only="status.mediaAttachments?.length > 0"
    )
    StatusCard(
      v-if="status.reblog"
      :status="status.reblog"
      border="~ rounded"
      :actions="false"
    )
    div(v-if="isDM")
"#;

    let (rest, ast) = parse(Span::new(input)).unwrap();

    let html_content = compile(&ast, &HsmlCompileOptions::default()).unwrap();

    assert_eq!(
        html_content,
        r#"<div class="space-y-3" :class="{
    'pt2 pb0.5 px3.5 bg-dm rounded-4 me--1': isDM,
    'ms--3.5 mt--1 ms--1': isDM && context !== 'details',
  }"><StatusBody v-if="(!isFiltered && isSensitiveNonSpoiler) || hideAllMedia" :status="status" :newer="newer" :with-action="!isDetails" :class="isDetails ? 'text-xl' : ''"></StatusBody><StatusSpoiler :enabled="hasSpoilerOrSensitiveMedia || isFiltered" :filter="isFiltered" :sensitive-non-spoiler="isSensitiveNonSpoiler || hideAllMedia" :is-d-m="isDM"><template v-if="spoilerTextPresent" #spoiler><p>{{ status.spoilerText }}</p></template><template v-else-if="filterPhrase" #spoiler><p>{{ `${$t('status.filter_hidden_phrase')}: ${filterPhrase}` }}</p></template><StatusBody v-if="!(isSensitiveNonSpoiler || hideAllMedia)" :status="status" :newer="newer" :with-action="!isDetails" :class="isDetails ? 'text-xl' : ''"></StatusBody><StatusTranslation :status="status"></StatusTranslation><StatusPoll v-if="status.poll" :status="status"></StatusPoll><StatusMedia v-if="status.mediaAttachments?.length" :status="status" :is-preview="isPreview"></StatusMedia><StatusPreviewCard v-if="status.card" :card="status.card" :small-picture-only="status.mediaAttachments?.length > 0"></StatusPreviewCard><StatusCard v-if="status.reblog" :status="status.reblog" border="~ rounded" :actions="false"></StatusCard><div v-if="isDM"></div></StatusSpoiler></div>"#
    );
    assert_eq!(*rest.fragment(), "");
}

// Void elements

#[test]
fn it_should_self_close_all_void_elements() {
    let void_elements = [
        "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source",
        "track", "wbr",
    ];

    for tag in void_elements {
        let input = format!("{tag}\n");
        let expected = format!("<{tag} />");
        assert_eq!(
            compile_content_core(&input),
            Ok(expected),
            "void element '{tag}' should self-close"
        );
    }
}

#[test]
fn it_should_self_close_void_elements_with_attributes() {
    assert_eq!(
        compile_content_core("img(src=\"photo.jpg\")\n"),
        Ok(String::from(r#"<img src="photo.jpg" />"#))
    );
    assert_eq!(
        compile_content_core("input(type=\"text\")\n"),
        Ok(String::from(r#"<input type="text" />"#))
    );
    assert_eq!(
        compile_content_core("meta(charset=\"utf-8\")\n"),
        Ok(String::from(r#"<meta charset="utf-8" />"#))
    );
    assert_eq!(
        compile_content_core("link(rel=\"stylesheet\" href=\"style.css\")\n"),
        Ok(String::from(
            r#"<link rel="stylesheet" href="style.css" />"#
        ))
    );
}

#[test]
fn it_should_not_self_close_non_void_elements() {
    assert_eq!(
        compile_content_core("div\n"),
        Ok(String::from("<div></div>"))
    );
    assert_eq!(
        compile_content_core("span\n"),
        Ok(String::from("<span></span>"))
    );
    assert_eq!(compile_content_core("p\n"), Ok(String::from("<p></p>")));
    assert_eq!(
        compile_content_core("section\n"),
        Ok(String::from("<section></section>"))
    );
}

#[test]
fn it_should_not_self_close_custom_components() {
    assert_eq!(
        compile_content_core("MyComponent\n"),
        Ok(String::from("<MyComponent></MyComponent>"))
    );
    assert_eq!(
        compile_content_core("StatusBody\n"),
        Ok(String::from("<StatusBody></StatusBody>"))
    );
}

// Class attribute merging

#[test]
fn it_should_merge_shorthand_and_attribute_classes() {
    assert_eq!(
        compile_content_core("div.foo(class=\"bar\") Hello\n"),
        Ok(String::from(r#"<div class="foo bar">Hello</div>"#))
    );
}

#[test]
fn it_should_merge_multiple_shorthand_with_attribute_classes() {
    assert_eq!(
        compile_content_core("div.a.b(class=\"c d\") Hello\n"),
        Ok(String::from(r#"<div class="a b c d">Hello</div>"#))
    );
}

#[test]
fn it_should_handle_class_attribute_without_shorthand() {
    assert_eq!(
        compile_content_core("div(class=\"foo bar\") Hello\n"),
        Ok(String::from(r#"<div class="foo bar">Hello</div>"#))
    );
}

#[test]
fn it_should_handle_shorthand_classes_without_attribute() {
    assert_eq!(
        compile_content_core("div.foo.bar Hello\n"),
        Ok(String::from(r#"<div class="foo bar">Hello</div>"#))
    );
}

#[test]
fn it_should_preserve_other_attributes_when_merging_classes() {
    assert_eq!(
        compile_content_core("div.foo(class=\"bar\" data-x=\"1\") Hello\n"),
        Ok(String::from(
            r#"<div class="foo bar" data-x="1">Hello</div>"#
        ))
    );
}

#[test]
fn it_should_merge_classes_with_shorthand_id() {
    assert_eq!(
        compile_content_core("div#app.foo(class=\"bar\") Hello\n"),
        Ok(String::from(r#"<div id="app" class="foo bar">Hello</div>"#))
    );
}

#[test]
fn it_should_merge_classes_and_preserve_framework_bindings() {
    assert_eq!(
        compile_content_core(
            "div.card(class=\"active\" :class=\"dynamicClass\" [class]=\"expr\")\n"
        ),
        Ok(String::from(
            r#"<div class="card active" :class="dynamicClass" [class]="expr"></div>"#
        ))
    );
}

#[test]
fn it_should_handle_valueless_class_with_shorthand() {
    // div.foo(class) — valueless class is a no-op, shorthand still works
    assert_eq!(
        compile_content_core("div.foo(class) Hello\n"),
        Ok(String::from(r#"<div class="foo">Hello</div>"#))
    );
}

#[test]
fn it_should_drop_valueless_class_without_shorthand() {
    // div(class) — valueless class with no shorthand produces no class attribute
    assert_eq!(
        compile_content_core("div(class) Hello\n"),
        Ok(String::from("<div>Hello</div>"))
    );
}

// Tests for compile_content error handling (mirrors lib.rs WASM logic)

#[test]
fn compile_content_should_return_html_for_valid_input() {
    let result = compile_content_core("h1 Hello World\n");
    assert_eq!(result, Ok(String::from("<h1>Hello World</h1>")));
}

#[test]
fn compile_content_should_return_html_for_valid_nested_input() {
    let result = compile_content_core("div\n  p Hello\n");
    assert_eq!(result, Ok(String::from("<div><p>Hello</p></div>")));
}

#[test]
fn compile_content_should_return_empty_html_for_empty_input() {
    let result = compile_content_core("");
    assert_eq!(result, Ok(String::from("")));
}

#[test]
fn it_should_compile_text_containing_double_slashes() {
    let result = compile_content_core("a Visit https://example.com\n");
    assert_eq!(
        result,
        Ok(String::from(r#"<a>Visit https://example.com</a>"#))
    );
}

#[test]
fn compile_content_diagnostics_should_return_html_for_valid_input() {
    let output = compile_content_diagnostics("h1 Hello\n").unwrap();
    assert_eq!(output.html, "<h1>Hello</h1>");
    assert!(output.diagnostics.is_empty());
}

#[test]
fn compile_content_diagnostics_should_return_warnings_for_duplicate_class() {
    let output = compile_content_diagnostics("h1.text-red.text-red Hello\n").unwrap();
    assert_eq!(output.html, r#"<h1 class="text-red text-red">Hello</h1>"#);
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].severity,
        hsml::diagnostic::Severity::Warning
    );
    assert_eq!(
        output.diagnostics[0].code,
        Some(ErrorCode::DuplicateClass.code().to_string())
    );
    assert_eq!(output.diagnostics[0].message, "Duplicate class 'text-red'");
}

#[test]
fn compile_content_diagnostics_should_return_diagnostics_for_invalid_input() {
    let result = compile_content_diagnostics("@@@invalid");
    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, hsml::diagnostic::Severity::Error);
    assert!(diagnostics[0].location.is_some());
}

#[test]
fn compile_content_diagnostics_should_return_warnings_for_duplicate_id() {
    let output = compile_content_diagnostics("div#a#b\n").unwrap();
    assert_eq!(output.html, r#"<div id="a"></div>"#);
    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].severity,
        hsml::diagnostic::Severity::Warning
    );
    assert_eq!(
        output.diagnostics[0].code,
        Some(ErrorCode::DuplicateId.code().to_string())
    );
    assert_eq!(
        output.diagnostics[0].message,
        "Duplicate id 'b' is not allowed"
    );
    assert!(output.diagnostics[0].location.is_some());
}

#[test]
fn compile_content_should_return_error_for_invalid_input() {
    let result = compile_content_core("123invalid");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("HSML parse error"));
}

#[test]
fn compile_content_should_return_error_for_special_characters() {
    let result = compile_content_core("@@@");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("HSML parse error"));
}

#[test]
fn it_should_error_on_unsupported_root_node_type() {
    let ast = RootNode {
        nodes: vec![HsmlNode::Text(TextNode {
            text: String::from("stray text"),
            is_block: false,
        })],
    };

    let result = compile(&ast, &HsmlCompileOptions::default());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported root node type"));
}

#[test]
fn it_should_error_on_unsupported_child_node_type() {
    let ast = RootNode {
        nodes: vec![HsmlNode::Tag(TagNode::without_location(
            "div",
            vec![],
            None,
            None,
            Some(TextNode {
                text: String::from("hello"),
                is_block: false,
            }),
            Some(vec![HsmlNode::Id(IdNode::new_without_location("stray"))]),
        ))],
    };

    let result = compile(&ast, &HsmlCompileOptions::default());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Unsupported child node type"));
    assert!(err.contains("<div>"));
}

#[test]
fn it_should_error_on_unsupported_attribute_node_type() {
    let ast = RootNode {
        nodes: vec![HsmlNode::Tag(TagNode::without_location(
            "span",
            vec![],
            None,
            Some(vec![HsmlNode::Class(ClassNode::new_without_location(
                "stray",
            ))]),
            None,
            None,
        ))],
    };

    let result = compile(&ast, &HsmlCompileOptions::default());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Unsupported node type in attributes"));
    assert!(err.contains("<span>"));
}

#[test]
fn it_should_compile_doctype_node() {
    let ast = RootNode {
        nodes: vec![HsmlNode::Doctype(DoctypeNode {
            doctype: String::from("html"),
        })],
    };

    let html_content = compile(&ast, &HsmlCompileOptions::default()).unwrap();

    assert_eq!(html_content, "<!DOCTYPE html>");
}

#[test]
fn it_should_compile_doctype_with_tags() {
    let input = "doctype html\nhtml\n  head\n  body\n";

    let result = compile_content_core(input);

    assert_eq!(
        result,
        Ok(String::from(
            "<!DOCTYPE html><html><head></head><body></body></html>"
        ))
    );
}

#[test]
fn it_should_compile_parsed_elk_main_content_component() {
    let input = r#"div(ref="container" :class="containerClass")
  .sticky.top-0.z10.backdrop-blur.native:lg:w-[calc(100vw-5rem)].native:xl:w-[calc(135%+(100vw-1200px)/2)](
    pt="[env(safe-area-inset-top,0)]"
    bg="[rgba(var(--rgb-bg-base),0.7)]"
  )
    .flex.justify-between.px5.py2.native:xl:flex(:class="{ 'xl:hidden': $route.name !== 'tag' }" border="b base")
      .flex.gap-3.items-center.py2.w-full(:overflow-hidden="!noOverflowHidden ? '' : false")
        NuxtLink.items-center.btn-text.p-0.xl:hidden(
          v-if="backOnSmallScreen || back"
          flex="~ gap1"
          :aria-label="$t('nav.back')"
          @click="$router.go(-1)"
        )
          .rtl-flip(i-ri:arrow-left-line)
        .flex.w-full.native-mac:justify-center.native-mac:text-center.native-mac:sm:justify-start(
          :truncate="!noOverflowHidden ? '' : false"
          data-tauri-drag-region
        )
          slot(name="title")
        .sm:hidde.nh-7.w-1px
      .flex.items-center.flex-shrink-0.gap-x-2
        slot(name="actions")
        PwaBadge.lg:hidden
        NavUser(v-if="isHydrated")
        NavUserSkeleton(v-else)
    slot(name="header")
      div(hidden)
  PwaInstallPrompt.lg:hidden
  .m-auto(:class="isHydrated && wideLayout ? 'xl:w-full sm:max-w-600px' : 'sm:max-w-600px md:shrink-0'")
    .h-6(hidden :class="{ 'xl:block': $route.name !== 'tag' && !$slots.header }")
    slot
"#;

    let (rest, ast) = parse(Span::new(input)).unwrap();

    let html_content = compile(&ast, &HsmlCompileOptions::default()).unwrap();

    assert_eq!(
        html_content,
        r#"<div ref="container" :class="containerClass"><div class="sticky top-0 z10 backdrop-blur native:lg:w-[calc(100vw-5rem)] native:xl:w-[calc(135%+(100vw-1200px)/2)]" pt="[env(safe-area-inset-top,0)]" bg="[rgba(var(--rgb-bg-base),0.7)]"><div class="flex justify-between px5 py2 native:xl:flex" :class="{ 'xl:hidden': $route.name !== 'tag' }" border="b base"><div class="flex gap-3 items-center py2 w-full" :overflow-hidden="!noOverflowHidden ? '' : false"><NuxtLink class="items-center btn-text p-0 xl:hidden" v-if="backOnSmallScreen || back" flex="~ gap1" :aria-label="$t('nav.back')" @click="$router.go(-1)"><div class="rtl-flip" i-ri:arrow-left-line></div></NuxtLink><div class="flex w-full native-mac:justify-center native-mac:text-center native-mac:sm:justify-start" :truncate="!noOverflowHidden ? '' : false" data-tauri-drag-region><slot name="title"></slot></div><div class="sm:hidde nh-7 w-1px"></div></div><div class="flex items-center flex-shrink-0 gap-x-2"><slot name="actions"></slot><PwaBadge class="lg:hidden"></PwaBadge><NavUser v-if="isHydrated"></NavUser><NavUserSkeleton v-else></NavUserSkeleton></div></div><slot name="header"><div hidden></div></slot></div><PwaInstallPrompt class="lg:hidden"></PwaInstallPrompt><div class="m-auto" :class="isHydrated && wideLayout ? 'xl:w-full sm:max-w-600px' : 'sm:max-w-600px md:shrink-0'"><div class="h-6" hidden :class="{ 'xl:block': $route.name !== 'tag' && !$slots.header }"></div><slot></slot></div></div>"#
    );
    assert_eq!(*rest.fragment(), "");
}

// Tests for compile output serialization (covers WASM JSON contract)

#[test]
fn compile_output_serializes_to_json() {
    let output = compile_content_diagnostics("h1 Hello\n").unwrap();
    let json = serde_json::to_value(&output).unwrap();

    assert_eq!(json["html"].as_str(), Some("<h1>Hello</h1>"));
    assert_eq!(json["diagnostics"].as_array().unwrap().len(), 0);
}

#[test]
fn compile_output_serializes_warnings_to_json() {
    let output = compile_content_diagnostics("h1.foo.foo Hello\n").unwrap();
    let json = serde_json::to_value(&output).unwrap();
    let diagnostics = json["diagnostics"].as_array().unwrap();

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["severity"].as_str(), Some("warning"));
    assert_eq!(
        diagnostics[0]["code"].as_str(),
        Some(ErrorCode::DuplicateClass.code())
    );
    assert_eq!(
        diagnostics[0]["message"].as_str(),
        Some("Duplicate class 'foo'")
    );
}

#[test]
fn compile_diagnostics_serialize_errors_to_json() {
    let diagnostics = compile_content_diagnostics("@@@invalid").unwrap_err();
    let json = serde_json::to_value(&diagnostics).unwrap();
    let arr = json.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["severity"].as_str(), Some("error"));
    assert_eq!(arr[0]["message"].as_str(), Some("parse error"));
}

#[test]
fn compile_diagnostics_serialize_duplicate_id_to_json() {
    let output = compile_content_diagnostics("div#a#b\n").unwrap();
    let json = serde_json::to_value(&output).unwrap();
    let diagnostics = json["diagnostics"].as_array().unwrap();

    assert_eq!(json["html"].as_str(), Some(r#"<div id="a"></div>"#));
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["severity"].as_str(), Some("warning"));
    assert_eq!(
        diagnostics[0]["code"].as_str(),
        Some(ErrorCode::DuplicateId.code())
    );
    assert_eq!(
        diagnostics[0]["message"].as_str(),
        Some("Duplicate id 'b' is not allowed")
    );
    assert!(diagnostics[0]["location"].is_object());
}

// Tests for check_content

#[test]
fn check_content_returns_empty_for_valid_input() {
    let diagnostics = check_content("h1 Hello\n");
    assert!(diagnostics.is_empty());
}

#[test]
fn check_content_returns_error_for_invalid_input() {
    let diagnostics = check_content("@@@invalid");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, hsml::diagnostic::Severity::Error);
}

#[test]
fn check_content_returns_warnings_for_duplicate_class() {
    let diagnostics = check_content("h1.foo.foo Hello\n");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].severity, hsml::diagnostic::Severity::Warning);
    assert_eq!(
        diagnostics[0].code,
        Some(ErrorCode::DuplicateClass.code().to_string())
    );
}

#[test]
fn check_content_does_not_compile() {
    // check_content should succeed even with valid input — it doesn't compile
    // This test verifies it only parses + validates, not compiles
    let diagnostics = check_content("div\n  span Hello\n");
    assert!(diagnostics.is_empty());
}

// Tests for format_content

#[test]
fn format_content_normalizes_indentation() {
    let opts = hsml::formatter::FormatOptions::default();
    let result = format_content_core("div\n    h1 Hello\n", &opts);
    assert_eq!(result.unwrap(), "div\n  h1 Hello\n");
}

#[test]
fn format_content_respects_indent_size() {
    let opts = hsml::formatter::FormatOptions {
        indent_size: 4,
        ..Default::default()
    };
    let result = format_content_core("div\n  h1 Hello\n", &opts);
    assert_eq!(result.unwrap(), "div\n    h1 Hello\n");
}

#[test]
fn format_content_returns_error_for_invalid_input() {
    let opts = hsml::formatter::FormatOptions::default();
    let result = format_content_core("@@@invalid", &opts);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("HSML parse error"));
}

// Pretty compilation tests

#[test]
fn compile_pretty_simple_nested() {
    let opts = HsmlCompileOptions {
        pretty: true,
        ..Default::default()
    };
    let (_, ast) = parse(Span::new("div\n  p Hello\n")).unwrap();
    assert_eq!(
        compile(&ast, &opts).unwrap(),
        "<div>\n  <p>Hello</p>\n</div>\n"
    );
}

#[test]
fn compile_pretty_deeply_nested() {
    let opts = HsmlCompileOptions {
        pretty: true,
        ..Default::default()
    };
    let (_, ast) = parse(Span::new("div\n  section\n    p Hello\n")).unwrap();
    assert_eq!(
        compile(&ast, &opts).unwrap(),
        "<div>\n  <section>\n    <p>Hello</p>\n  </section>\n</div>\n"
    );
}

#[test]
fn compile_pretty_void_elements() {
    let opts = HsmlCompileOptions {
        pretty: true,
        ..Default::default()
    };
    let (_, ast) = parse(Span::new("div\n  br\n  hr\n")).unwrap();
    assert_eq!(
        compile(&ast, &opts).unwrap(),
        "<div>\n  <br />\n  <hr />\n</div>\n"
    );
}

#[test]
fn compile_pretty_with_doctype() {
    let opts = HsmlCompileOptions {
        pretty: true,
        ..Default::default()
    };
    let (_, ast) = parse(Span::new(
        "doctype html\nhtml\n  head\n  body\n    p Hello\n",
    ))
    .unwrap();
    assert_eq!(
        compile(&ast, &opts).unwrap(),
        "<!DOCTYPE html>\n<html>\n  <head></head>\n  <body>\n    <p>Hello</p>\n  </body>\n</html>\n"
    );
}

#[test]
fn compile_pretty_with_comments() {
    let opts = HsmlCompileOptions {
        pretty: true,
        ..Default::default()
    };
    let (_, ast) = parse(Span::new("div\n  //! hello\n  p World\n")).unwrap();
    assert_eq!(
        compile(&ast, &opts).unwrap(),
        "<div>\n  <!-- hello -->\n  <p>World</p>\n</div>\n"
    );
}

#[test]
fn compile_pretty_with_custom_indent() {
    let opts = HsmlCompileOptions {
        pretty: true,
        indent_size: 4,
    };
    let (_, ast) = parse(Span::new("div\n  p Hello\n")).unwrap();
    assert_eq!(
        compile(&ast, &opts).unwrap(),
        "<div>\n    <p>Hello</p>\n</div>\n"
    );
}

#[test]
fn compile_pretty_inline_text_no_extra_newline() {
    let opts = HsmlCompileOptions {
        pretty: true,
        ..Default::default()
    };
    let (_, ast) = parse(Span::new("p Hello World\n")).unwrap();
    assert_eq!(compile(&ast, &opts).unwrap(), "<p>Hello World</p>\n");
}

#[test]
fn compile_pretty_via_diagnostics_api() {
    let opts = HsmlCompileOptions {
        pretty: true,
        ..Default::default()
    };
    let output = compile_content_diagnostics_with_options("div\n  p Hello\n", &opts).unwrap();
    assert_eq!(output.html, "<div>\n  <p>Hello</p>\n</div>\n");
    assert!(output.diagnostics.is_empty());
}
