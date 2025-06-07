/// A simple struct to hold the byte‐offset and character for each position in the input string.
struct CharPos {
    byte_idx: usize,
    ch: char,
}

/// Represents a detected `SELECT` keyword: its character‐index in `chars` and its parenthesis‐`depth` at that point.
struct FoundSelect {
    char_idx: usize,
    depth: usize,
}

fn main() {
    // Example deeply nested SQL string:
    let sql = r#"
        SELECT id, name
        FROM (
            SELECT u.id, u.name
            FROM (
                SELECT id, name FROM users WHERE active = 1
            ) AS u
            WHERE u.name LIKE 'A%'
        ) AS t
        WHERE t.id > 100;
    "#;

    // 1) Build a Vec<CharPos> so we know both chars and their byte offsets in the original `sql`.
    let mut chars: Vec<CharPos> = Vec::new();
    for (byte_idx, ch) in sql.char_indices() {
        chars.push(CharPos { byte_idx, ch });
    }
    let n = chars.len();

    // 2) Create an uppercase version of each character for keyword detection (case‐insensitive).
    let mut upper_chars: Vec<char> = Vec::with_capacity(n);
    for cp in &chars {
        for uc in cp.ch.to_uppercase() {
            // `to_uppercase()` can yield multiple chars for some Unicode, but SQL keywords are ASCII here.
            upper_chars.push(uc);
            break;
        }
    }

    // 3) We will scan once, tracking:
    //    - `depth[i]`: how many unmatched '(' are open _after_ processing chars[i].
    //    - States for in‐single‐quote, in‐double‐quote, in‐line‐comment, in‐block‐comment.
    let mut depth = vec![0usize; n];
    let mut current_depth: usize = 0;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    // We also want to record every time we see a valid "SELECT" and what the `depth` was at that position.
    let mut found_selects: Vec<FoundSelect> = Vec::new();

    let mut i = 0;
    while i < n {
        let cp = &chars[i];
        let c = cp.ch;

        // FIRST: handle exiting line‐comments or block‐comments if needed
        if in_line_comment {
            // in a `-- comment to end of line`
            if c == '\n' {
                in_line_comment = false;
            }
            // While in a line comment, we ignore all SQL syntax (no parens, no SELECT detection)
            depth[i] = current_depth;
            i += 1;
            continue;
        }
        if in_block_comment {
            // in a `/* ... */` comment
            if c == '*' && (i + 1 < n) && chars[i + 1].ch == '/' {
                // exit block comment on "*/"
                in_block_comment = false;
                // Advance over both '*' and '/'
                depth[i] = current_depth;
                i += 1;
                if i < n {
                    // record depth at the '/' as well
                    depth[i] = current_depth;
                    i += 1;
                }
                continue;
            }
            // still in block comment
            depth[i] = current_depth;
            i += 1;
            continue;
        }

        // SECOND: handle entering line comment or block comment if we see "--" or "/*"
        if !in_single && !in_double {
            // detect start of line comment
            if c == '-' && (i + 1 < n) && chars[i + 1].ch == '-' {
                in_line_comment = true;
                // record depth at '-' and move on
                depth[i] = current_depth;
                i += 1;
                if i < n {
                    depth[i] = current_depth; // record for second '-'
                    i += 1;
                }
                continue;
            }
            // detect start of block comment
            if c == '/' && (i + 1 < n) && chars[i + 1].ch == '*' {
                in_block_comment = true;
                depth[i] = current_depth;
                i += 1;
                if i < n {
                    depth[i] = current_depth;
                    i += 1;
                }
                continue;
            }
        }

        // THIRD: handle entering/exiting single or double quotes (string literals)
        if !in_double && !in_line_comment && !in_block_comment && c == '\'' {
            // Toggle single‐quote state. We do NOT try to handle escaped single‐quotes (e.g. `''`) robustly here,
            // but in most SQLite code, doubling '' is how you escape. If you need full SQL‐standard escape handling,
            // you'd have to peek ahead. For simplicity we toggle.
            in_single = !in_single;
            depth[i] = current_depth;
            i += 1;
            continue;
        }
        if !in_single && !in_line_comment && !in_block_comment && c == '"' {
            // Toggle double‐quote state (used for identifiers in SQLite).
            in_double = !in_double;
            depth[i] = current_depth;
            i += 1;
            continue;
        }

        // FOURTH: If we're inside a single‐ or double‐quoted literal, skip any syntax logic.
        if in_single || in_double {
            // Everything inside quotes is ignored for parentheses or SELECT detection.
            depth[i] = current_depth;
            i += 1;
            continue;
        }

        // FIFTH: We are not in any comment or string. Now handle '(' and ')'
        if c == '(' {
            current_depth += 1;
        } else if c == ')' {
            if current_depth > 0 {
                current_depth -= 1;
            }
        }
        // Record the depth _after_ processing this character:
        depth[i] = current_depth;

        // SIXTH: Check for the keyword "SELECT" at position `i`, only if not in a comment/string.
        // We require that the next 6 uppercase chars are "SELECT", and that they are
        // token‐bounded (non‐alphanumeric or underscore on either side).
        if upper_chars[i] == 'S' {
            if i + 6 <= n {
                let slice: String = upper_chars[i..i + 6].iter().collect();
                if slice == "SELECT" {
                    // Check boundary before `i`:
                    let ok_before = if i == 0 {
                        true
                    } else {
                        let prev = upper_chars[i - 1];
                        !prev.is_ascii_alphanumeric() && prev != '_'
                    };
                    // Check boundary after `i + 5`:
                    let ok_after = if i + 6 >= n {
                        true
                    } else {
                        let next = upper_chars[i + 6];
                        !next.is_ascii_alphanumeric() && next != '_'
                    };
                    if ok_before && ok_after {
                        // We have a stand‐alone "SELECT" token:
                        found_selects.push(FoundSelect {
                            char_idx: i,
                            depth: depth[i],
                        });
                    }
                }
            }
        }

        i += 1;
    }

    // 4) Now, for each found "SELECT", find its end position.
    //    If depth > 0, we assume it's inside parentheses. We look for the first index j > start
    //    where `depth[j] == (start_depth - 1)` AND chars[j].ch == ')'. That marks the closing
    //    parenthesis of that subquery. If depth == 0, it's the top‐level SELECT; we look for ';' or end‐of‐string.
    let mut extracted_subqueries: Vec<String> = Vec::new();

    for f in &found_selects {
        let start_idx = f.char_idx;
        let start_depth = f.depth;

        // Determine the byte offset in the original `sql` string where this subquery begins:
        let byte_start = chars[start_idx].byte_idx;

        // Find the end‐byte:
        let byte_end = if start_depth > 0 {
            // Look for the matching closing ')' that brings us back to depth = start_depth - 1
            let mut byte_end_offset = sql.len(); // fallback: end of string
            for j in (start_idx + 1)..n {
                if depth[j] == (start_depth - 1) && chars[j].ch == ')' {
                    byte_end_offset = chars[j].byte_idx; // do NOT include the ')'
                    break;
                }
            }
            byte_end_offset
        } else {
            // Top‐level SELECT: search for the first semicolon after start_idx
            let mut byte_end_offset = sql.len();
            for j in (start_idx + 6)..n {
                // look at the raw character
                if chars[j].ch == ';' {
                    byte_end_offset = chars[j].byte_idx;
                    break;
                }
            }
            byte_end_offset
        };

        // Extract the substring from byte_start up to (but not including) byte_end:
        let subquery_str = sql[byte_start..byte_end].trim().to_string();
        extracted_subqueries.push(subquery_str);
    }

    // 5) Print them out (in discovery order):
    println!("Found {} subquery(ies):\n", extracted_subqueries.len());
    for (i, sq) in extracted_subqueries.iter().enumerate() {
        println!("--- Subquery #{} ---\n{}\n", i + 1, sq);
    }
}
