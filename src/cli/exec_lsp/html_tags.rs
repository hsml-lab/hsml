/// HTML tag documentation for hover display.
pub struct HtmlTagInfo {
    pub description: &'static str,
    pub mdn_url: &'static str,
}

pub fn lookup(tag: &str) -> Option<HtmlTagInfo> {
    let info = match tag {
        // Main root
        "html" => HtmlTagInfo {
            description: "The html element represents the root (top-level element) of an HTML document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/html",
        },

        // Document metadata
        "head" => HtmlTagInfo {
            description: "The head element contains machine-readable information (metadata) about the document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/head",
        },
        "title" => HtmlTagInfo {
            description: "The title element defines the document's title that is shown in a browser's title bar or a page's tab.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/title",
        },
        "base" => HtmlTagInfo {
            description: "The base element specifies the base URL to use for all relative URLs in a document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/base",
        },
        "link" => HtmlTagInfo {
            description: "The link element specifies relationships between the current document and an external resource.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/link",
        },
        "meta" => HtmlTagInfo {
            description: "The meta element represents metadata that cannot be represented by other HTML meta-related elements.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meta",
        },
        "style" => HtmlTagInfo {
            description: "The style element contains CSS styling information for a document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/style",
        },

        // Sectioning root
        "body" => HtmlTagInfo {
            description: "The body element represents the content of an HTML document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/body",
        },

        // Content sectioning
        "article" => HtmlTagInfo {
            description: "The article element represents a self-contained composition in a document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/article",
        },
        "aside" => HtmlTagInfo {
            description: "The aside element represents a portion of a document whose content is only indirectly related to the main content.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside",
        },
        "footer" => HtmlTagInfo {
            description: "The footer element represents a footer for its nearest sectioning content or sectioning root element.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/footer",
        },
        "header" => HtmlTagInfo {
            description: "The header element represents introductory content or a set of navigational links.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/header",
        },
        "h1" => HtmlTagInfo {
            description: "The h1 element represents a level 1 section heading.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements",
        },
        "h2" => HtmlTagInfo {
            description: "The h2 element represents a level 2 section heading.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements",
        },
        "h3" => HtmlTagInfo {
            description: "The h3 element represents a level 3 section heading.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements",
        },
        "h4" => HtmlTagInfo {
            description: "The h4 element represents a level 4 section heading.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements",
        },
        "h5" => HtmlTagInfo {
            description: "The h5 element represents a level 5 section heading.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements",
        },
        "h6" => HtmlTagInfo {
            description: "The h6 element represents a level 6 section heading.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/Heading_Elements",
        },
        "hgroup" => HtmlTagInfo {
            description: "The hgroup element represents a heading grouped with any secondary content.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hgroup",
        },
        "main" => HtmlTagInfo {
            description: "The main element represents the dominant content of the body of a document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/main",
        },
        "nav" => HtmlTagInfo {
            description: "The nav element represents a section of a page whose purpose is to provide navigation links.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/nav",
        },
        "section" => HtmlTagInfo {
            description: "The section element represents a generic standalone section of a document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/section",
        },
        "search" => HtmlTagInfo {
            description: "The search element represents a part of the document that contains form controls or content related to performing a search.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/search",
        },

        // Text content
        "blockquote" => HtmlTagInfo {
            description: "The blockquote element indicates that the enclosed text is an extended quotation.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote",
        },
        "dd" => HtmlTagInfo {
            description: "The dd element provides the description, definition, or value for the preceding term in a description list.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dd",
        },
        "div" => HtmlTagInfo {
            description: "The div element is the generic container for flow content. It has no effect on the content or layout until styled in some way using CSS.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div",
        },
        "dl" => HtmlTagInfo {
            description: "The dl element represents a description list of groups of terms and descriptions.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl",
        },
        "dt" => HtmlTagInfo {
            description: "The dt element specifies a term in a description or definition list.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dt",
        },
        "figcaption" => HtmlTagInfo {
            description: "The figcaption element represents a caption or legend describing the rest of the contents of its parent figure element.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figcaption",
        },
        "figure" => HtmlTagInfo {
            description: "The figure element represents self-contained content, potentially with an optional caption.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figure",
        },
        "hr" => HtmlTagInfo {
            description: "The hr element represents a thematic break between paragraph-level elements.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr",
        },
        "li" => HtmlTagInfo {
            description: "The li element is used to represent an item in a list.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li",
        },
        "ol" => HtmlTagInfo {
            description: "The ol element represents an ordered list of items.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ol",
        },
        "p" => HtmlTagInfo {
            description: "The p element represents a paragraph.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p",
        },
        "pre" => HtmlTagInfo {
            description: "The pre element represents preformatted text which is to be presented exactly as written in the HTML file.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre",
        },
        "ul" => HtmlTagInfo {
            description: "The ul element represents an unordered list of items.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/ul",
        },
        "menu" => HtmlTagInfo {
            description: "The menu element is a semantic alternative to ul for representing an unordered list of items.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/menu",
        },

        // Inline text semantics
        "a" => HtmlTagInfo {
            description: "The a element, with its href attribute, creates a hyperlink to web pages, files, email addresses, or anything else a URL can address.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a",
        },
        "abbr" => HtmlTagInfo {
            description: "The abbr element represents an abbreviation or acronym.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/abbr",
        },
        "b" => HtmlTagInfo {
            description: "The b element is used to draw attention to text without indicating that it's of extra importance.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/b",
        },
        "br" => HtmlTagInfo {
            description: "The br element produces a line break in text.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/br",
        },
        "code" => HtmlTagInfo {
            description: "The code element displays its contents styled in a fashion intended to indicate that the text is a short fragment of computer code.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code",
        },
        "em" => HtmlTagInfo {
            description: "The em element marks text that has stress emphasis.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em",
        },
        "i" => HtmlTagInfo {
            description: "The i element represents a range of text that is set off from the normal text for some reason.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/i",
        },
        "kbd" => HtmlTagInfo {
            description: "The kbd element represents a span of inline text denoting textual user input from a keyboard or other input device.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/kbd",
        },
        "mark" => HtmlTagInfo {
            description: "The mark element represents text which is marked or highlighted for reference or notation purposes.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/mark",
        },
        "q" => HtmlTagInfo {
            description: "The q element indicates that the enclosed text is a short inline quotation.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q",
        },
        "s" => HtmlTagInfo {
            description: "The s element renders text with a strikethrough. Use it to represent things that are no longer relevant or accurate.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/s",
        },
        "small" => HtmlTagInfo {
            description: "The small element represents side-comments and small print, like copyright and legal text.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/small",
        },
        "span" => HtmlTagInfo {
            description: "The span element is a generic inline container for phrasing content, which does not inherently represent anything.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span",
        },
        "strong" => HtmlTagInfo {
            description: "The strong element indicates that its contents have strong importance, seriousness, or urgency.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong",
        },
        "sub" => HtmlTagInfo {
            description: "The sub element specifies inline text which should be displayed as subscript.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub",
        },
        "sup" => HtmlTagInfo {
            description: "The sup element specifies inline text which should be displayed as superscript.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup",
        },
        "time" => HtmlTagInfo {
            description: "The time element represents a specific period in time.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/time",
        },
        "u" => HtmlTagInfo {
            description: "The u element represents a span of inline text which should be rendered with a non-textual annotation.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u",
        },

        // Image and multimedia
        "audio" => HtmlTagInfo {
            description: "The audio element is used to embed sound content in documents.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio",
        },
        "img" => HtmlTagInfo {
            description: "The img element embeds an image into the document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img",
        },
        "video" => HtmlTagInfo {
            description: "The video element embeds a media player which supports video playback into the document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video",
        },
        "source" => HtmlTagInfo {
            description: "The source element specifies multiple media resources for the picture, audio, or video elements.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/source",
        },
        "picture" => HtmlTagInfo {
            description: "The picture element contains zero or more source elements and one img element to offer alternative versions of an image.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/picture",
        },
        "track" => HtmlTagInfo {
            description: "The track element is used as a child of the media elements audio and video to specify timed text tracks.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/track",
        },

        // Embedded content
        "iframe" => HtmlTagInfo {
            description: "The iframe element represents a nested browsing context, embedding another HTML page into the current one.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/iframe",
        },
        "embed" => HtmlTagInfo {
            description: "The embed element embeds external content at the specified point in the document.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/embed",
        },
        "object" => HtmlTagInfo {
            description: "The object element represents an external resource, which can be treated as an image, a nested browsing context, or a resource to be handled by a plugin.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/object",
        },

        // SVG and MathML
        "svg" => HtmlTagInfo {
            description: "The svg element is a container for SVG graphics.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/SVG/Element/svg",
        },
        "math" => HtmlTagInfo {
            description: "The math element is the top-level MathML element, used to write a single mathematical formula.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/MathML/Element/math",
        },
        "canvas" => HtmlTagInfo {
            description: "The canvas element is used to draw graphics via scripting (usually JavaScript).",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas",
        },

        // Scripting
        "script" => HtmlTagInfo {
            description: "The script element is used to embed executable code or data, typically JavaScript.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/script",
        },
        "noscript" => HtmlTagInfo {
            description: "The noscript element defines a section of HTML to be inserted if a script type on the page is unsupported or scripting is currently turned off.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/noscript",
        },

        // Table content
        "table" => HtmlTagInfo {
            description: "The table element represents tabular data.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/table",
        },
        "caption" => HtmlTagInfo {
            description: "The caption element specifies the caption (or title) of a table.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/caption",
        },
        "thead" => HtmlTagInfo {
            description: "The thead element defines a set of rows defining the head of the columns of the table.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/thead",
        },
        "tbody" => HtmlTagInfo {
            description: "The tbody element encapsulates a set of table rows, indicating that they comprise the body of the table.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tbody",
        },
        "tfoot" => HtmlTagInfo {
            description: "The tfoot element defines a set of rows summarizing the columns of the table.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tfoot",
        },
        "tr" => HtmlTagInfo {
            description: "The tr element defines a row of cells in a table.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr",
        },
        "td" => HtmlTagInfo {
            description: "The td element defines a cell of a table that contains data.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td",
        },
        "th" => HtmlTagInfo {
            description: "The th element defines a cell as the header of a group of table cells.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/th",
        },
        "col" => HtmlTagInfo {
            description: "The col element defines one or more columns in a column group represented by its parent colgroup element.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/col",
        },
        "colgroup" => HtmlTagInfo {
            description: "The colgroup element defines a group of columns within a table.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/colgroup",
        },

        // Forms
        "form" => HtmlTagInfo {
            description: "The form element represents a document section containing interactive controls for submitting information.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/form",
        },
        "input" => HtmlTagInfo {
            description: "The input element is used to create interactive controls for web-based forms to accept data from the user.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input",
        },
        "button" => HtmlTagInfo {
            description: "The button element is an interactive element activated by a user with a mouse, keyboard, finger, voice command, or other assistive technology.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button",
        },
        "select" => HtmlTagInfo {
            description: "The select element represents a control that provides a menu of options.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/select",
        },
        "option" => HtmlTagInfo {
            description: "The option element is used to define an item contained in a select, optgroup, or datalist element.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/option",
        },
        "optgroup" => HtmlTagInfo {
            description: "The optgroup element creates a grouping of options within a select element.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/optgroup",
        },
        "textarea" => HtmlTagInfo {
            description: "The textarea element represents a multi-line plain-text editing control.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/textarea",
        },
        "label" => HtmlTagInfo {
            description: "The label element represents a caption for an item in a user interface.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/label",
        },
        "fieldset" => HtmlTagInfo {
            description: "The fieldset element is used to group several controls as well as labels within a web form.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/fieldset",
        },
        "legend" => HtmlTagInfo {
            description: "The legend element represents a caption for the content of its parent fieldset.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/legend",
        },
        "datalist" => HtmlTagInfo {
            description: "The datalist element contains a set of option elements that represent the permissible or recommended options available in other controls.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/datalist",
        },
        "output" => HtmlTagInfo {
            description: "The output element is a container element into which a site or app can inject the results of a calculation or user action.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/output",
        },
        "progress" => HtmlTagInfo {
            description: "The progress element displays an indicator showing the completion progress of a task.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/progress",
        },
        "meter" => HtmlTagInfo {
            description: "The meter element represents either a scalar value within a known range or a fractional value.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/meter",
        },

        // Interactive elements
        "details" => HtmlTagInfo {
            description: "The details element creates a disclosure widget in which information is visible only when toggled open.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details",
        },
        "summary" => HtmlTagInfo {
            description: "The summary element specifies a summary, caption, or legend for a details element's disclosure box.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/summary",
        },
        "dialog" => HtmlTagInfo {
            description: "The dialog element represents a dialog box or other interactive component, such as a dismissible alert or subwindow.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dialog",
        },

        // Web components
        "template" => HtmlTagInfo {
            description: "The template element is used to hold HTML that is not to be rendered immediately when a page is loaded but may be instantiated subsequently during runtime.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/template",
        },
        "slot" => HtmlTagInfo {
            description: "The slot element is a placeholder inside a web component that you can fill with your own markup.",
            mdn_url: "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/slot",
        },

        _ => return None,
    };
    Some(info)
}
