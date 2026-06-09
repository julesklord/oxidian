//! Pure text-processing helpers for markdown content parsing.
//! Snippet extraction, title extraction, date parsing, and word counting.

/// Derive a title-cased title from a filename slug (without extension).
/// Example: `career-tracks` -> `Career Tracks`
pub fn slug_to_title(stem: &str) -> String {
    stem.split('-')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{}{}", upper, chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract H1 title from first non-empty line of the note body (skipping frontmatter).
pub fn extract_h1_title(content: &str) -> Option<String> {
    let body = strip_frontmatter(content);
    let title = first_non_empty_line(body).and_then(markdown_h1_text)?;
    non_empty_trimmed(&strip_markdown_chars(title)).map(str::to_string)
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn first_non_empty_line(value: &str) -> Option<&str> {
    value.lines().map(str::trim).find(|line| !line.is_empty())
}

fn markdown_h1_text(line: &str) -> Option<&str> {
    line.strip_prefix("# ").and_then(non_empty_trimmed)
}

/// Extract display title for a note.
/// Priority: H1 on first non-empty line -> frontmatter `title:` -> filename slug.
pub fn extract_title(fm_title: Option<&str>, content: &str, filename: &str) -> String {
    if let Some(h1) = extract_h1_title(content) {
        return h1;
    }
    if let Some(title) = fm_title {
        if !title.is_empty() {
            return title.to_string();
        }
    }
    let stem = filename.strip_suffix(".md").unwrap_or(filename);
    slug_to_title(stem)
}

/// Remove YAML frontmatter from Markdown content.
pub fn strip_frontmatter(content: &str) -> &str {
    let Some(rest) = content.strip_prefix("---") else {
        return content;
    };
    match rest.find("\n---") {
        Some(end) => {
            let after = end + 4;
            rest[after..].trim_start()
        }
        None => content,
    }
}

/// Check if a line is useful for snippet extraction.
fn is_snippet_line(line: &str) -> bool {
    let t = line.trim();
    !t.is_empty() && !t.starts_with('#') && !t.starts_with("```") && !t.starts_with("---")
}

/// Extract sub-heading text stripped of '#' prefix.
fn extract_subheading_text(line: &str) -> Option<&str> {
    let t = line.trim();
    let stripped = t.trim_start_matches('#');
    if stripped.len() < t.len() && stripped.starts_with(' ') {
        let text = stripped.trim();
        if !text.is_empty() {
            return Some(text);
        }
    }
    None
}

/// Strip leading list markers from a line.
fn strip_list_marker(line: &str) -> &str {
    let t = line.trim_start();
    strip_unordered_marker(t)
        .or_else(|| strip_ordered_marker(t))
        .unwrap_or(t)
}

fn strip_unordered_marker(s: &str) -> Option<&str> {
    ["* ", "- ", "+ "]
        .iter()
        .find_map(|prefix| s.strip_prefix(prefix))
}

fn strip_ordered_marker(s: &str) -> Option<&str> {
    let dot_pos = s.find(". ")?;
    if dot_pos <= 3 && s[..dot_pos].chars().all(|c| c.is_ascii_digit()) {
        Some(&s[dot_pos + 2..])
    } else {
        None
    }
}

fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let mut idx = max_len;
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    format!("{}...", &s[..idx])
}

/// Count the number of words in the note body (excluding frontmatter and H1).
pub fn count_body_words(content: &str) -> u32 {
    let without_fm = strip_frontmatter(content);
    let body = without_h1_line(without_fm).unwrap_or(without_fm);
    body.split_whitespace()
        .filter(|w| {
            !w.chars()
                .all(|c| matches!(c, '#' | '*' | '_' | '`' | '~' | '-' | '>' | '|'))
        })
        .count() as u32
}

/// Extract a snippet: first ~160 chars of plain text content.
pub fn extract_snippet(content: &str) -> String {
    let without_fm = strip_frontmatter(content);
    let body = without_h1_line(without_fm).unwrap_or(without_fm);
    let clean: String = body
        .lines()
        .filter(|line| is_snippet_line(line))
        .map(strip_list_marker)
        .collect::<Vec<&str>>()
        .join(" ");
    let stripped = strip_markdown_chars(&clean);
    let trimmed = stripped.trim();
    if !trimmed.is_empty() {
        return truncate_with_ellipsis(trimmed, 160);
    }
    let heading_text: String = body
        .lines()
        .filter_map(extract_subheading_text)
        .collect::<Vec<&str>>()
        .join(" ");
    let heading_trimmed = strip_markdown_chars(&heading_text);
    let heading_trimmed = heading_trimmed.trim();
    if heading_trimmed.is_empty() {
        return String::new();
    }
    truncate_with_ellipsis(heading_trimmed, 160)
}

fn without_h1_line(s: &str) -> Option<&str> {
    let mut offset = 0;
    for line in s.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']).trim();
        if trimmed.starts_with("# ") {
            return Some(&s[offset + line.len()..]);
        }
        if !trimmed.is_empty() {
            return None;
        }
        offset += line.len();
    }
    None
}

fn is_markdown_formatting(ch: char) -> bool {
    matches!(ch, '*' | '_' | '`' | '~')
}

fn strip_markdown_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '[' if chars.peek() == Some(&'[') => {
                chars.next();
                let mut inner = String::new();
                while let Some(c) = chars.next() {
                    if c == ']' && chars.peek() == Some(&']') {
                        chars.next();
                        break;
                    }
                    inner.push(c);
                }
                let display = inner
                    .find('|')
                    .map_or(inner.as_str(), |idx| &inner[idx + 1..]);
                result.push_str(display);
            }
            '[' => {
                let mut inner = String::new();
                for c in chars.by_ref() {
                    if c == ']' {
                        break;
                    }
                    inner.push(c);
                }
                if chars.peek() == Some(&'(') {
                    chars.next();
                    for c in chars.by_ref() {
                        if c == ')' {
                            break;
                        }
                    }
                    result.push_str(&inner);
                } else {
                    result.push('[');
                    result.push_str(&inner);
                    result.push(']');
                }
            }
            c if is_markdown_formatting(c) => {}
            _ => result.push(ch),
        }
    }
    result
}

/// Updates the `title:` field in YAML frontmatter.
/// If frontmatter does not exist, it creates one at the top of the file.
pub fn update_frontmatter_title(content: &str, new_title: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Check if we have frontmatter at all
    if lines.first().map(|s| s.trim()) == Some("---") {
        let mut title_index = None;
        let mut closing_index = None;

        for (i, line) in lines.iter().enumerate().skip(1) {
            let trimmed = line.trim();
            if trimmed == "---" {
                closing_index = Some(i);
                break;
            }
            if trimmed.starts_with("title:") {
                title_index = Some(i);
            }
        }

        if let Some(idx) = title_index {
            lines[idx] = format!("title: \"{}\"", new_title.replace('"', "\\\""));
        } else if let Some(idx) = closing_index {
            lines.insert(
                idx,
                format!("title: \"{}\"", new_title.replace('"', "\\\"")),
            );
        }

        lines.join("\n")
    } else {
        let mut new_content = format!("---\ntitle: \"{}\"\n---\n", new_title.replace('"', "\\\""));
        new_content.push_str(content);
        new_content
    }
}
