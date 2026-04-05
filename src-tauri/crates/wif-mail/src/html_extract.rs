//! Minimal HTML-to-plain-text extraction with no external dependencies.

/// Convert an HTML string to readable plain text.
///
/// Steps applied in order:
/// 1. Remove `<script>` and `<style>` blocks (including their content).
/// 2. Replace block-level tags (`<br>`, `<p>`, `<div>`, `<li>`, `<tr>`) with newlines.
/// 3. Strip all remaining HTML tags.
/// 4. Decode common HTML entities (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`, `&nbsp;`).
/// 5. Collapse runs of whitespace / blank lines.
pub fn html_to_text(html: &str) -> String {
    let step1 = remove_blocks(html);
    let step2 = replace_block_tags(&step1);
    let step3 = strip_tags(&step2);
    let step4 = decode_entities(&step3);
    collapse_whitespace(&step4)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Remove `<script>…</script>` and `<style>…</style>` blocks.
fn remove_blocks(input: &str) -> String {
    remove_tag_block(
        &remove_tag_block(input, "script"),
        "style",
    )
}

/// Remove all occurrences of `<tag_name …>…</tag_name>` (case-insensitive).
fn remove_tag_block(input: &str, tag_name: &str) -> String {
    let open = format!("<{tag_name}");
    let close = format!("</{tag_name}>");
    let lower = input.to_lowercase();
    let open_lower = open.to_lowercase();
    let close_lower = close.to_lowercase();

    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let lower_bytes = lower.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    while i < len {
        if lower_bytes[i..].starts_with(open_lower.as_bytes()) {
            // Find the closing tag.
            let search_from = i + open_lower.len();
            if let Some(rel) = lower[search_from..].find(close_lower.as_str()) {
                i = search_from + rel + close_lower.len();
            } else {
                // No closing tag found — skip to end.
                break;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Replace common block-level / line-break tags with a newline character.
fn replace_block_tags(input: &str) -> String {
    const BLOCK_TAGS: &[&str] = &[
        "<br", "<BR",
        "<p>", "<P>", "</p>", "</P>",
        "<div", "<DIV", "</div>", "</DIV>",
        "<li>", "<LI>", "</li>", "</LI>",
        "<tr>", "<TR>", "</tr>", "</TR>",
        "<h1", "<h2", "<h3", "<h4", "<h5", "<h6",
        "<H1", "<H2", "<H3", "<H4", "<H5", "<H6",
    ];

    let mut result = input.to_string();
    for tag in BLOCK_TAGS {
        result = result.replace(tag, &format!("\n{tag}"));
    }
    result
}

/// Remove all remaining HTML tags (anything matching `<…>`).
fn strip_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut inside = false;

    for ch in input.chars() {
        match ch {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => result.push(ch),
            _ => {}
        }
    }

    result
}

/// Decode a small set of common HTML entities.
fn decode_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// Collapse runs of whitespace and trim blank lines.
fn collapse_whitespace(input: &str) -> String {
    // Normalise CRLF to LF.
    let normalised = input.replace("\r\n", "\n").replace('\r', "\n");

    // Collapse each line: trim and reduce internal whitespace runs to one space.
    let lines: Vec<String> = normalised
        .lines()
        .map(|line| {
            line.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    // Remove runs of consecutive blank lines (keep at most one).
    let mut result = String::with_capacity(normalised.len());
    let mut last_blank = false;

    for line in &lines {
        if line.is_empty() {
            if !last_blank {
                result.push('\n');
            }
            last_blank = true;
        } else {
            result.push_str(line);
            result.push('\n');
            last_blank = false;
        }
    }

    result.trim().to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_simple_tags() {
        let input = "<p>Hello, <b>world</b>!</p>";
        assert_eq!(html_to_text(input), "Hello, world!");
    }

    #[test]
    fn removes_script_blocks() {
        let input = "Before<script>alert('xss')</script>After";
        assert_eq!(html_to_text(input), "BeforeAfter");
    }

    #[test]
    fn removes_style_blocks() {
        let input = "Before<style>.foo{color:red}</style>After";
        assert_eq!(html_to_text(input), "BeforeAfter");
    }

    #[test]
    fn decodes_entities() {
        let input = "&lt;html&gt; &amp; &quot;quoted&quot; &#39;apos&#39;";
        let result = html_to_text(input);
        assert!(result.contains('<'));
        assert!(result.contains('>'));
        assert!(result.contains('&'));
        assert!(result.contains('"'));
        assert!(result.contains('\''));
    }

    #[test]
    fn collapses_whitespace() {
        let input = "<p>   lots   of   space   </p>";
        assert_eq!(html_to_text(input), "lots of space");
    }

    #[test]
    fn inserts_newlines_for_br_tags() {
        let input = "line1<br>line2<br/>line3";
        let result = html_to_text(input);
        // After processing, "line1", "line2", "line3" should be on separate lines.
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.iter().any(|l| l.contains("line1")));
        assert!(lines.iter().any(|l| l.contains("line2")));
        assert!(lines.iter().any(|l| l.contains("line3")));
    }

    #[test]
    fn empty_input_returns_empty_string() {
        assert_eq!(html_to_text(""), "");
    }

    #[test]
    fn plain_text_passthrough() {
        let input = "Just plain text with no tags.";
        assert_eq!(html_to_text(input), input);
    }
}
