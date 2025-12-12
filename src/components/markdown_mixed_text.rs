use iocraft::prelude::*;

/// Render a subset of Markdown into an iocraft `MixedText`.
///
/// Notes / limitations:
/// - This intentionally supports a conservative subset (paragraphs, emphasis/strong, inline code,
///   links, headings, lists, blockquotes, code blocks).
/// - Anything unknown falls back to plain text.
/// - iocraft `MixedText` is single-style-per-span; we approximate nested styles by composing spans.
#[derive(Default, Props)]
pub struct MarkdownMixedTextProps {
    pub content: String,
    /// Soft-wrap content.
    pub wrap: TextWrap,
}

#[component]
pub fn MarkdownMixedText(
    _hooks: Hooks,
    props: &MarkdownMixedTextProps,
) -> impl Into<AnyElement<'static>> {
    let contents = render_markdown_to_mixed_text_contents(&props.content);

    element! {
        MixedText(contents: contents, wrap: props.wrap)
    }
}

fn push_text(out: &mut Vec<MixedTextContent>, s: &str) {
    if s.is_empty() {
        return;
    }
    out.push(MixedTextContent::new(s));
}

fn render_markdown_to_mixed_text_contents(input: &str) -> Vec<MixedTextContent> {
    // Use the `markdown` crate to parse to an mdast.
    // We avoid enabling extra features; default CommonMark parsing is sufficient.
    let mut out: Vec<MixedTextContent> = Vec::new();

    let parse_result = markdown::to_mdast(input, &markdown::ParseOptions::default());
    let Ok(mdast) = parse_result else {
        // Fallback: plain text
        push_text(&mut out, input);
        return out;
    };

    render_node(&mdast, &mut out, StyleState::default());
    out
}

#[derive(Clone, Copy, Default)]
struct StyleState {
    bold: bool,
    italic: bool,
    code: bool,
}

fn apply_style(mut c: MixedTextContent, st: StyleState) -> MixedTextContent {
    if st.bold {
        c = c.weight(Weight::Bold);
    }
    // iocraft MixedTextContent doesn't currently expose italic/background in this repo's version.
    // Approximate emphasis with a subtle color.
    if st.italic {
        c = c.color(Color::AnsiValue(250));
    }
    if st.code {
        // Approximate inline code with a muted color.
        c = c.color(Color::AnsiValue(244));
    }
    c
}

fn push_styled_text(out: &mut Vec<MixedTextContent>, s: &str, st: StyleState) {
    if s.is_empty() {
        return;
    }
    out.push(apply_style(MixedTextContent::new(s), st));
}

fn render_children(
    children: &[markdown::mdast::Node],
    out: &mut Vec<MixedTextContent>,
    st: StyleState,
) {
    for child in children {
        render_node(child, out, st);
    }
}

fn render_node(node: &markdown::mdast::Node, out: &mut Vec<MixedTextContent>, st: StyleState) {
    use markdown::mdast::Node;

    match node {
        Node::Root(r) => render_children(&r.children, out, st),

        Node::Text(t) => push_styled_text(out, &t.value, st),

        Node::Emphasis(e) => {
            let mut st2 = st;
            st2.italic = true;
            render_children(&e.children, out, st2);
        }

        Node::Strong(s) => {
            let mut st2 = st;
            st2.bold = true;
            render_children(&s.children, out, st2);
        }

        Node::InlineCode(c) => {
            let mut st2 = st;
            st2.code = true;
            push_styled_text(out, &c.value, st2);
        }

        Node::Code(c) => {
            // Fenced/indented code block: preserve newlines.
            // Add a leading newline if we're not already at line start.
            if !out.is_empty() {
                push_text(out, "\n");
            }
            let mut code_style = st;
            code_style.code = true;
            // Render as one span to preserve spacing better.
            out.push(
                apply_style(MixedTextContent::new(&c.value), code_style)
                    .color(Color::AnsiValue(250)),
            );
            push_text(out, "\n");
        }

        Node::Break(_) => push_text(out, "\n"),
        Node::ThematicBreak(_) => push_text(out, "\n---\n"),

        Node::Paragraph(p) => {
            render_children(&p.children, out, st);
            push_text(out, "\n");
        }

        Node::Heading(h) => {
            // Render heading as bold with a newline.
            let mut st2 = st;
            st2.bold = true;
            render_children(&h.children, out, st2);
            push_text(out, "\n");
        }

        Node::Blockquote(bq) => {
            // Prefix lines with "> ".
            push_text(out, "> ");
            render_children(&bq.children, out, st);
            push_text(out, "\n");
        }

        Node::List(list) => {
            for (idx, item) in list.children.iter().enumerate() {
                match item {
                    Node::ListItem(li) => {
                        let bullet = if list.ordered {
                            format!("{}. ", idx + 1)
                        } else {
                            "- ".to_string()
                        };
                        push_text(out, &bullet);
                        render_children(&li.children, out, st);
                        push_text(out, "\n");
                    }
                    _ => render_node(item, out, st),
                }
            }
        }

        Node::Link(link) => {
            // Render link text underlined-ish via color, then include destination in parens.
            let before_len = out.len();
            render_children(&link.children, out, st);
            // Apply link color to the spans just added.
            for c in out.iter_mut().skip(before_len) {
                *c = c.clone().color(Color::Rgb {
                    r: 100,
                    g: 149,
                    b: 237,
                });
            }
            if !link.url.is_empty() {
                push_text(out, " (");
                push_text(out, &link.url);
                push_text(out, ")");
            }
        }

        Node::Image(img) => {
            // Render alt + url.
            if !img.alt.is_empty() {
                push_text(out, &img.alt);
            } else {
                push_text(out, "[image]");
            }
            if !img.url.is_empty() {
                push_text(out, " (");
                push_text(out, &img.url);
                push_text(out, ")");
            }
        }

        // Tables/etc: fall back to plain string where possible.
        _ => {
            // As a conservative fallback, we try to use `to_string` debug-ish.
            // Better than dropping content.
            push_text(out, &format!("{}", node_to_plain(node)));
        }
    }
}

fn node_to_plain(node: &markdown::mdast::Node) -> String {
    use markdown::mdast::Node;
    match node {
        Node::Text(t) => t.value.clone(),
        Node::InlineCode(c) => c.value.clone(),
        Node::Code(c) => c.value.clone(),
        Node::Break(_) => "\n".to_string(),
        _ => "".to_string(),
    }
}
