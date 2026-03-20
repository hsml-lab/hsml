use hsml::{
    compile_content_core,
    compiler::{HsmlCompileOptions, compile},
    parser::{
        HsmlNode, RootNode, Span, class::node::ClassNode, doctype::node::DoctypeNode,
        id::node::IdNode, parse::parse, tag::node::TagNode, text::node::TextNode,
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
        nodes: vec![HsmlNode::Tag(TagNode {
            tag: String::from("h1"),
            id: None,
            classes: None,
            attributes: None,
            text: Some(TextNode {
                text: String::from("Hello World"),
            }),
            children: None,
        })],
    };

    let html_content = compile(&ast, &HsmlCompileOptions::default()).unwrap();

    assert_eq!(html_content, "<h1>Hello World</h1>");
}

#[test]
fn it_should_compile_content_with_id() {
    let ast = RootNode {
        nodes: vec![HsmlNode::Tag(TagNode {
            tag: String::from("h1"),
            id: Some(IdNode {
                id: String::from("title"),
            }),
            classes: None,
            attributes: None,
            text: Some(TextNode {
                text: String::from("Hello World"),
            }),
            children: None,
        })],
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
        r#"<h1 class="text-red">Vite CJS Faker Demo</h1><div class="card"><div class="card__image"><img :src="natureImageUrl" :alt="'Background image for ' + fullName"/></div><div class="card__profile"><img :src="avatarUrl" :alt="'Avatar image of ' + fullName"/></div><div class="card__body">{{ fullName }}</div></div>"#
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
        r#"<!-- test comment on root layer --><figure class="md:flex bg-slate-100 rounded-xl p-8 md:p-0 dark:bg-slate-800/10"><!-- test comment --><img class="w-24 h-24 md:w-48 md:h-auto md:rounded-none rounded-full mx-auto" src="/fancy-avatar.jpg" alt="" width="384" height="512"/><div class="pt-6 md:p-8 text-center md:text-left space-y-4"><blockquote v-if="showBlockquote"><p class="text-lg font-medium">"Tailwind CSS is the only framework that I've seen scale
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
  }"><StatusBody v-if="(!isFiltered && isSensitiveNonSpoiler) || hideAllMedia" :status="status" :newer="newer" :with-action="!isDetails" :class="isDetails ? 'text-xl' : ''"/><StatusSpoiler :enabled="hasSpoilerOrSensitiveMedia || isFiltered" :filter="isFiltered" :sensitive-non-spoiler="isSensitiveNonSpoiler || hideAllMedia" :is-d-m="isDM"><template v-if="spoilerTextPresent" #spoiler><p>{{ status.spoilerText }}</p></template><template v-else-if="filterPhrase" #spoiler><p>{{ `${$t('status.filter_hidden_phrase')}: ${filterPhrase}` }}</p></template><StatusBody v-if="!(isSensitiveNonSpoiler || hideAllMedia)" :status="status" :newer="newer" :with-action="!isDetails" :class="isDetails ? 'text-xl' : ''"/><StatusTranslation :status="status"/><StatusPoll v-if="status.poll" :status="status"/><StatusMedia v-if="status.mediaAttachments?.length" :status="status" :is-preview="isPreview"/><StatusPreviewCard v-if="status.card" :card="status.card" :small-picture-only="status.mediaAttachments?.length > 0"/><StatusCard v-if="status.reblog" :status="status.reblog" border="~ rounded" :actions="false"/><div v-if="isDM"/></StatusSpoiler></div>"#
    );
    assert_eq!(*rest.fragment(), "");
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
        })],
    };

    let result = compile(&ast, &HsmlCompileOptions::default());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported root node type"));
}

#[test]
fn it_should_error_on_unsupported_child_node_type() {
    let ast = RootNode {
        nodes: vec![HsmlNode::Tag(TagNode {
            tag: String::from("div"),
            id: None,
            classes: None,
            attributes: None,
            text: Some(TextNode {
                text: String::from("hello"),
            }),
            children: Some(vec![HsmlNode::Id(IdNode {
                id: String::from("stray"),
            })]),
        })],
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
        nodes: vec![HsmlNode::Tag(TagNode {
            tag: String::from("span"),
            id: None,
            classes: None,
            attributes: Some(vec![HsmlNode::Class(ClassNode {
                name: String::from("stray"),
            })]),
            text: None,
            children: None,
        })],
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
        Ok(String::from("<!DOCTYPE html><html><head/><body/></html>"))
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
        r#"<div ref="container" :class="containerClass"><div class="sticky top-0 z10 backdrop-blur native:lg:w-[calc(100vw-5rem)] native:xl:w-[calc(135%+(100vw-1200px)/2)]" pt="[env(safe-area-inset-top,0)]" bg="[rgba(var(--rgb-bg-base),0.7)]"><div class="flex justify-between px5 py2 native:xl:flex" :class="{ 'xl:hidden': $route.name !== 'tag' }" border="b base"><div class="flex gap-3 items-center py2 w-full" :overflow-hidden="!noOverflowHidden ? '' : false"><NuxtLink class="items-center btn-text p-0 xl:hidden" v-if="backOnSmallScreen || back" flex="~ gap1" :aria-label="$t('nav.back')" @click="$router.go(-1)"><div class="rtl-flip" i-ri:arrow-left-line/></NuxtLink><div class="flex w-full native-mac:justify-center native-mac:text-center native-mac:sm:justify-start" :truncate="!noOverflowHidden ? '' : false" data-tauri-drag-region><slot name="title"/></div><div class="sm:hidde nh-7 w-1px"/></div><div class="flex items-center flex-shrink-0 gap-x-2"><slot name="actions"/><PwaBadge class="lg:hidden"/><NavUser v-if="isHydrated"/><NavUserSkeleton v-else/></div></div><slot name="header"><div hidden/></slot></div><PwaInstallPrompt class="lg:hidden"/><div class="m-auto" :class="isHydrated && wideLayout ? 'xl:w-full sm:max-w-600px' : 'sm:max-w-600px md:shrink-0'"><div class="h-6" hidden :class="{ 'xl:block': $route.name !== 'tag' && !$slots.header }"/><slot/></div></div>"#
    );
    assert_eq!(*rest.fragment(), "");
}
