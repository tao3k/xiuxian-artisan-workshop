use std::fmt::Write as _;

use comrak::{
    Arena, Options,
    nodes::{AstNode, ListType, NodeValue},
    parse_document,
};

use super::escape::{
    escape_markdown_v2_code, escape_markdown_v2_text, escape_markdown_v2_url,
    normalize_code_fence_language, trim_trailing_blank_lines,
};

struct MarkdownV2Renderer {
    rendered: String,
    ordered_list_stack: Vec<usize>,
    list_is_ordered_stack: Vec<bool>,
    link_stack: Vec<String>,
    in_code_block: bool,
    in_table: bool,
    in_table_head: bool,
    table_cell_index: usize,
    table_headers: Vec<String>,
    current_table_row: Vec<String>,
    current_table_cell: String,
}

impl MarkdownV2Renderer {
    fn new() -> Self {
        Self {
            rendered: String::new(),
            ordered_list_stack: Vec::new(),
            list_is_ordered_stack: Vec::new(),
            link_stack: Vec::new(),
            in_code_block: false,
            in_table: false,
            in_table_head: false,
            table_cell_index: 0,
            table_headers: Vec::new(),
            current_table_row: Vec::new(),
            current_table_cell: String::new(),
        }
    }

    fn render(mut self, markdown: &str) -> String {
        let arena = Arena::new();
        let options = telegram_comrak_options();
        let root = parse_document(&arena, markdown, &options);

        for child in root.children() {
            self.render_node(child);
        }

        self.finish(markdown)
    }

    fn render_node<'a>(&mut self, node: &'a AstNode<'a>) {
        self.handle_start_node(node);
        for child in node.children() {
            self.render_node(child);
        }
        self.handle_end_node(node);
    }

    fn handle_start_node<'a>(&mut self, node: &'a AstNode<'a>) {
        match &node.data().value {
            NodeValue::Strong | NodeValue::Heading(_) => self.push_fragment("*"),
            NodeValue::Emph => self.push_fragment("_"),
            NodeValue::Strikethrough => self.push_fragment("~"),
            NodeValue::CodeBlock(block) => self.start_code_block(&block.info, &block.literal),
            NodeValue::Link(link) => self.start_link(link.url.clone()),
            NodeValue::List(list) => self.start_list(list.list_type, list.start),
            NodeValue::Item(_) => self.start_list_item(),
            NodeValue::TaskItem(task) => {
                self.start_list_item();
                self.push_task_list_marker(task.symbol.is_some());
            }
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) => self.push_fragment("> "),
            NodeValue::Table(_) => self.start_table(),
            NodeValue::TableRow(is_head) => self.start_table_row(*is_head),
            NodeValue::TableCell => self.start_table_cell(),
            NodeValue::Text(text) => self.push_text(text),
            NodeValue::Code(code) => self.push_inline_code(&code.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => self.rendered.push('\n'),
            NodeValue::ThematicBreak => self.rendered.push_str("\n\\-\\-\\-\\-\n"),
            NodeValue::HtmlBlock(html) => self.push_text(&html.literal),
            NodeValue::HtmlInline(html) => self.push_text(html),
            NodeValue::FootnoteReference(reference) => {
                self.push_footnote_reference(&reference.name);
            }
            _ => {}
        }
    }

    fn handle_end_node<'a>(&mut self, node: &'a AstNode<'a>) {
        match &node.data().value {
            NodeValue::Strong => self.push_fragment("*"),
            NodeValue::Emph => self.push_fragment("_"),
            NodeValue::Strikethrough => self.push_fragment("~"),
            NodeValue::CodeBlock(_) => self.end_code_block(),
            NodeValue::Link(_) => self.end_link(),
            NodeValue::Heading(_) => self.rendered.push_str("*\n\n"),
            NodeValue::Paragraph => self.rendered.push_str("\n\n"),
            NodeValue::List(_) => self.end_list(),
            NodeValue::TableCell => self.end_table_cell(),
            NodeValue::TableRow(_) => self.end_table_row(),
            NodeValue::Table(_) => self.end_table(),
            _ => {}
        }
    }

    fn start_table(&mut self) {
        self.in_table = true;
        self.in_table_head = false;
        self.table_headers.clear();
        self.current_table_row.clear();
        self.current_table_cell.clear();
        if !self.rendered.is_empty() && !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
    }

    fn start_table_row(&mut self, is_head: bool) {
        self.in_table_head = is_head;
        self.table_cell_index = 0;
        self.current_table_row.clear();
    }

    fn start_table_cell(&mut self) {
        self.table_cell_index += 1;
        self.current_table_cell.clear();
    }

    fn end_table_cell(&mut self) {
        if self.in_table && self.table_cell_index > 0 {
            self.current_table_row
                .push(self.current_table_cell.trim().to_string());
            self.current_table_cell.clear();
        }
    }

    fn end_table_row(&mut self) {
        if self.in_table_head {
            self.table_headers = self.current_table_row.clone();
            self.in_table_head = false;
        } else if !self.current_table_row.is_empty() {
            self.rendered.push_str("• ");
            self.rendered
                .push_str(&self.render_table_row_as_bullet(&self.current_table_row));
            self.rendered.push('\n');
        }

        self.current_table_row.clear();
        self.current_table_cell.clear();
        self.table_cell_index = 0;
    }

    fn end_table(&mut self) {
        self.in_table = false;
        self.in_table_head = false;
        self.table_headers.clear();
        self.current_table_row.clear();
        self.current_table_cell.clear();
        self.table_cell_index = 0;
        if !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
    }

    fn start_code_block(&mut self, info: &str, literal: &str) {
        self.in_code_block = true;
        self.rendered.push_str("```");
        if let Some(language) = normalize_code_fence_language(info) {
            self.rendered.push_str(&language);
        }
        self.rendered.push('\n');
        self.push_fragment(&escape_markdown_v2_code(literal));
    }

    fn end_code_block(&mut self) {
        self.in_code_block = false;
        if !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
        self.rendered.push_str("```\n\n");
    }

    fn start_link(&mut self, target: String) {
        self.push_fragment("[");
        self.link_stack.push(target);
    }

    fn end_link(&mut self) {
        let link_target = self.link_stack.pop().unwrap_or_default();
        self.push_fragment("]");
        self.push_fragment("(");
        self.push_fragment(&escape_markdown_v2_url(&link_target));
        self.push_fragment(")");
    }

    fn start_list(&mut self, list_type: ListType, start: usize) {
        if matches!(list_type, ListType::Ordered) {
            let start_number = u64::try_from(start).unwrap_or(u64::MAX);
            self.ordered_list_stack
                .push(usize::try_from(start_number).unwrap_or(usize::MAX));
            self.list_is_ordered_stack.push(true);
        } else {
            self.ordered_list_stack.push(1);
            self.list_is_ordered_stack.push(false);
        }
    }

    fn end_list(&mut self) {
        self.ordered_list_stack.pop();
        self.list_is_ordered_stack.pop();
        self.rendered.push('\n');
    }

    fn start_list_item(&mut self) {
        if !self.rendered.is_empty() && !self.rendered.ends_with('\n') {
            self.rendered.push('\n');
        }
        match self.list_is_ordered_stack.last().copied() {
            Some(true) => {
                if let Some(current) = self.ordered_list_stack.last_mut() {
                    let _ = write!(self.rendered, "{current}\\. ");
                    *current += 1;
                } else {
                    self.rendered.push_str("• ");
                }
            }
            _ => self.rendered.push_str("• "),
        }
    }

    fn push_text(&mut self, text: &str) {
        if self.in_code_block {
            self.push_fragment(&escape_markdown_v2_code(text));
        } else {
            self.push_fragment(&escape_markdown_v2_text(text));
        }
    }

    fn push_inline_code(&mut self, code: &str) {
        self.push_fragment("`");
        self.push_fragment(&escape_markdown_v2_code(code));
        self.push_fragment("`");
    }

    fn push_task_list_marker(&mut self, checked: bool) {
        if checked {
            self.rendered.push_str("\\[x\\] ");
        } else {
            self.rendered.push_str("\\[ \\] ");
        }
    }

    fn push_footnote_reference(&mut self, name: &str) {
        self.push_fragment("\\[");
        self.push_fragment(&escape_markdown_v2_text(name));
        self.push_fragment("\\]");
    }

    fn push_fragment(&mut self, fragment: &str) {
        if self.in_table && self.table_cell_index > 0 {
            self.current_table_cell.push_str(fragment);
        } else {
            self.rendered.push_str(fragment);
        }
    }

    fn render_table_row_as_bullet(&self, row: &[String]) -> String {
        if !self.table_headers.is_empty() && self.table_headers.len() == row.len() {
            return self
                .table_headers
                .iter()
                .zip(row.iter())
                .map(|(header, value)| format!("{header}: {value}"))
                .collect::<Vec<_>>()
                .join(" \\| ");
        }

        if row.len() == 2 {
            return format!("{}: {}", row[0], row[1]);
        }

        row.join(" \\| ")
    }

    fn finish(mut self, markdown: &str) -> String {
        trim_trailing_blank_lines(&mut self.rendered);
        if self.rendered.is_empty() {
            escape_markdown_v2_text(markdown)
        } else {
            self.rendered
        }
    }
}

fn telegram_comrak_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.tasklist = true;
    options
}

/// Render Markdown as Telegram `MarkdownV2` text.
#[must_use]
pub fn markdown_to_telegram_markdown_v2(markdown: &str) -> String {
    MarkdownV2Renderer::new().render(markdown)
}
