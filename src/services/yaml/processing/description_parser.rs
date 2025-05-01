pub fn parse_description(content: &str) -> Option<String> {
    let mut lines = content.lines();
    // 1) Must start with '---'
    if lines.next()? != "---" {
        return None;
    }

    let mut in_block = false;
    let mut desc = String::new();

    for line in lines {
        let trimmed = line.trim_start();

        // 2) if we hit the end of front-matter, stop
        if trimmed == "---" {
            break;
        }

        if !in_block {
            // 3) look for the `description:` key at top-level
            if let Some(rest) = trimmed.strip_prefix("description:") {
                let rest = rest.trim();
                match rest.chars().next() {
                    // block scalar start
                    Some('|') | Some('>') => {
                        in_block = true;
                        continue;
                    }
                    // inline scalar on the same line
                    _ if !rest.is_empty() => {
                        // strip optional quotes
                        let s = rest.trim_matches('"').to_string();
                        return Some(s);
                    }
                    // exactly `description:` with no value → treat as block
                    _ => {
                        in_block = true;
                        continue;
                    }
                }
            }
        } else {
            // 4) we're inside a block — collect indented lines
            // YAML spec: block-scalar content must be indented at least one space
            if line.starts_with(' ') || line.starts_with('\t') {
                // drop only the leading indent
                desc.push_str(line.trim_start());
                desc.push('\n');
            } else {
                // non-indented → end of block
                break;
            }
        }
    }

    if desc.is_empty() {
        None
    } else {
        // trim the final newline
        Some(desc.trim_end().to_string())
    }
}