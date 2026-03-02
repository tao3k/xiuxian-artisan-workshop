use std::fmt::Write;

use crate::link_graph::{LinkGraphDisplayHit, LinkGraphPlannedSearchPayload};

/// Render planned search payload into lean XML-Lite consumed by LLM tool context.
pub(super) fn render_xml_lite(payload: &LinkGraphPlannedSearchPayload) -> String {
    let mut output = String::new();
    for hit in &payload.hits {
        render_hit(&mut output, hit);
    }
    output
}

fn render_hit(output: &mut String, hit: &LinkGraphDisplayHit) {
    let title = select_title(hit);
    let content = if hit.best_section.trim().is_empty() {
        title.to_string()
    } else {
        format!("{title} | section: {}", hit.best_section.trim())
    };
    let hit_type = infer_hit_type(hit);
    let _ = writeln!(
        output,
        "  <hit id=\"{}\" score=\"{:.4}\" type=\"{}\">{}</hit>",
        escape_xml_attr(&hit.path),
        hit.score,
        hit_type,
        escape_xml_text(content.trim())
    );
}

fn infer_hit_type(hit: &LinkGraphDisplayHit) -> &'static str {
    if let Some(mapped) = hit
        .doc_type
        .as_deref()
        .and_then(infer_hit_type_from_kind_value)
    {
        return mapped;
    }

    if let Some(mapped) = infer_hit_type_from_tags(&hit.tags) {
        return mapped;
    }

    let path = hit.path.to_ascii_lowercase();
    let title = hit.title.to_ascii_lowercase();
    if path.contains("/agenda/") || path.contains("agenda") || title.contains("agenda") {
        return "agenda";
    }
    if path.contains("/journal/") || path.contains("journal") || title.contains("journal") {
        return "journal";
    }
    if path.contains("/tasks/")
        || path.contains("/task/")
        || path.contains("todo")
        || title.contains("task")
    {
        return "task";
    }
    if !has_markdown_extension(&hit.path) {
        return "attachment";
    }
    "note"
}

fn has_markdown_extension(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
}

fn infer_hit_type_from_tags(tags: &[String]) -> Option<&'static str> {
    for raw in tags {
        if let Some(mapped) = infer_hit_type_from_kind_value(raw) {
            return Some(mapped);
        }
    }
    None
}

fn infer_hit_type_from_kind_value(raw: &str) -> Option<&'static str> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "agenda" | "schedule" | "calendar" => Some("agenda"),
        "journal" | "diary" | "reflection" => Some("journal"),
        "task" | "tasks" | "todo" => Some("task"),
        "attachment" | "asset" | "image" | "pdf" | "file" => Some("attachment"),
        "doc" | "note" | "knowledge" => Some("note"),
        _ => {
            if normalized.contains("agenda")
                || normalized.contains("schedule")
                || normalized.contains("calendar")
            {
                return Some("agenda");
            }
            if normalized.contains("journal")
                || normalized.contains("diary")
                || normalized.contains("reflection")
            {
                return Some("journal");
            }
            if normalized.contains("task") || normalized.contains("todo") {
                return Some("task");
            }
            if normalized.contains("attachment")
                || normalized.contains("asset")
                || normalized.contains("image")
                || normalized.contains("pdf")
                || normalized.contains("file")
            {
                return Some("attachment");
            }
            if normalized.contains("doc")
                || normalized.contains("note")
                || normalized.contains("knowledge")
            {
                return Some("note");
            }
            None
        }
    }
}

fn select_title(hit: &LinkGraphDisplayHit) -> &str {
    let title = hit.title.trim();
    if title.is_empty() {
        hit.stem.trim()
    } else {
        title
    }
}

fn escape_xml_attr(input: &str) -> String {
    escape_xml(input, true)
}

fn escape_xml_text(input: &str) -> String {
    escape_xml(input, false)
}

fn escape_xml(input: &str, escape_quotes: bool) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' if escape_quotes => out.push_str("&quot;"),
            '\'' if escape_quotes => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}
