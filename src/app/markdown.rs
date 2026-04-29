use crate::storage::models::{List, Status, Task};

pub trait Markdown {
    fn markdown(&self) -> String;
}

impl Markdown for List {
    fn markdown(&self) -> String {
        format!("# {}\n", self.name)
    }
}

impl Markdown for Task {
    fn markdown(&self) -> String {
        let mut out = String::new();
        write_task(&mut out, self, 0);
        out
    }
}

fn write_task(out: &mut String, task: &Task, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    out.push_str(&format!(
        "{}- [{}] {}\n",
        indent,
        if task.status == Status::Completed {
            "x"
        } else {
            " "
        },
        task.title
    ));
    for sub in &task.sub_tasks {
        write_task(out, sub, indent_level + 1);
    }
}

// --- Import ----------------------------------------------------------------

/// Result of parsing a markdown document into something the importer can
/// materialize. `name` is the first H1 the parser saw (or `None` if the input
/// only had bullets / lower-level headings).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportedList {
    pub name: Option<String>,
    pub tasks: Vec<ImportedTask>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImportedTask {
    pub title: String,
    pub completed: bool,
    pub children: Vec<ImportedTask>,
}

/// Parse a markdown document into a list-and-tasks tree. The parser is
/// intentionally permissive — it accepts both checkbox bullets (`- [ ] x`)
/// and plain bullets (`- x`, `* x`, `+ x`, `1. x`), uses the first `#` as
/// the list name, and treats every `##`/deeper heading as a top-level
/// parent task whose children are the bullets that follow it (until the
/// next heading at the same or higher level).
pub fn parse_import(input: &str) -> ImportedList {
    let mut out = ImportedList::default();

    // Each frame is (level, path-of-indices-into-out.tasks). The base frame
    // and any heading-section frame both use level `-1` so the first bullet
    // (level 0) doesn't pop them; deeper nesting works by frame.level >=
    // bullet.level.
    let mut stack: Vec<(i32, Vec<usize>)> = vec![(-1, vec![])];

    for raw_line in input.lines() {
        let line = raw_line.trim_end_matches(['\r']);
        if line.trim().is_empty() {
            continue;
        }

        // ---- headings ----
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let mut level = 1usize;
            let mut rest = rest;
            while let Some(r) = rest.strip_prefix('#') {
                rest = r;
                level += 1;
            }
            // Require a space after the hashes — otherwise it's not a
            // heading (e.g. a URL fragment in a bullet).
            if let Some(text) = rest.strip_prefix(' ') {
                let text = text.trim();
                if level == 1 && out.name.is_none() {
                    out.name = Some(text.to_string());
                    stack = vec![(-1, vec![])];
                    continue;
                }
                // ## or deeper, or a second H1: parent task.
                let parent = ImportedTask {
                    title: text.to_string(),
                    completed: false,
                    children: vec![],
                };
                out.tasks.push(parent);
                let parent_idx = out.tasks.len() - 1;
                stack = vec![(-1, vec![]), (-1, vec![parent_idx])];
                continue;
            }
        }

        // ---- bullets ----
        let Some((indent_spaces, content, completed)) = parse_bullet(raw_line) else {
            continue;
        };
        let level = (indent_spaces / 2) as i32;

        while stack.len() > 1 && stack.last().map(|f| f.0).unwrap_or(-1) >= level {
            stack.pop();
        }

        let new_task = ImportedTask {
            title: content,
            completed,
            children: vec![],
        };

        let frame_path = stack.last().expect("stack always has a base").1.clone();
        let target = locate_children(&mut out.tasks, &frame_path);
        target.push(new_task);
        let new_idx = target.len() - 1;

        let mut new_path = frame_path;
        new_path.push(new_idx);
        stack.push((level, new_path));
    }

    out
}

fn locate_children<'a>(
    top: &'a mut Vec<ImportedTask>,
    path: &[usize],
) -> &'a mut Vec<ImportedTask> {
    if path.is_empty() {
        return top;
    }
    let mut current = &mut top[path[0]];
    for &i in &path[1..] {
        current = &mut current.children[i];
    }
    &mut current.children
}

/// Recognise a bullet line. Returns `(indent_in_spaces, content, completed)`.
/// Tabs count as 4 spaces.
fn parse_bullet(line: &str) -> Option<(usize, String, bool)> {
    let mut indent = 0usize;
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' => {
                indent += 1;
                chars.next();
            }
            '\t' => {
                indent += 4;
                chars.next();
            }
            _ => break,
        }
    }
    let rest: String = chars.collect();

    let after_marker = if let Some(r) = rest.strip_prefix("- ") {
        r
    } else if let Some(r) = rest.strip_prefix("* ") {
        r
    } else if let Some(r) = rest.strip_prefix("+ ") {
        r
    } else {
        // numbered list: leading digits, then ". "
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        let after = &rest[digits.len()..];
        let after = after.strip_prefix('.').or_else(|| after.strip_prefix(')'))?;
        after.strip_prefix(' ')?
    };

    // Optional checkbox.
    let (completed, content) = if let Some(rest) = after_marker.strip_prefix("[ ] ") {
        (false, rest.to_string())
    } else if let Some(rest) = after_marker
        .strip_prefix("[x] ")
        .or_else(|| after_marker.strip_prefix("[X] "))
    {
        (true, rest.to_string())
    } else if after_marker == "[ ]" {
        (false, String::new())
    } else if after_marker == "[x]" || after_marker == "[X]" {
        (true, String::new())
    } else {
        (false, after_marker.to_string())
    };

    let content = content.trim().to_string();
    if content.is_empty() {
        return None;
    }
    Some((indent, content, completed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_h1_as_list_name() {
        let md = "# My List\n- one\n- two\n";
        let out = parse_import(md);
        assert_eq!(out.name.as_deref(), Some("My List"));
        assert_eq!(out.tasks.len(), 2);
        assert_eq!(out.tasks[0].title, "one");
    }

    #[test]
    fn h2_becomes_parent_task() {
        let md = "# Top\n## Section A\n- a1\n- a2\n## Section B\n- b1\n";
        let out = parse_import(md);
        assert_eq!(out.name.as_deref(), Some("Top"));
        assert_eq!(out.tasks.len(), 2);
        assert_eq!(out.tasks[0].title, "Section A");
        assert_eq!(out.tasks[0].children.len(), 2);
        assert_eq!(out.tasks[0].children[1].title, "a2");
        assert_eq!(out.tasks[1].title, "Section B");
        assert_eq!(out.tasks[1].children[0].title, "b1");
    }

    #[test]
    fn checkbox_states_round_trip() {
        let md = "- [x] done\n- [ ] todo\n- [X] also done\n";
        let out = parse_import(md);
        assert_eq!(out.tasks.len(), 3);
        assert!(out.tasks[0].completed);
        assert!(!out.tasks[1].completed);
        assert!(out.tasks[2].completed);
    }

    #[test]
    fn indented_bullets_become_children() {
        let md = "- parent\n  - child\n    - grandchild\n- sibling\n";
        let out = parse_import(md);
        assert_eq!(out.tasks.len(), 2);
        assert_eq!(out.tasks[0].children.len(), 1);
        assert_eq!(out.tasks[0].children[0].title, "child");
        assert_eq!(out.tasks[0].children[0].children[0].title, "grandchild");
        assert_eq!(out.tasks[1].title, "sibling");
    }

    #[test]
    fn plain_bullets_without_checkbox_count_as_tasks() {
        let md = "* foo\n+ bar\n1. baz\n";
        let out = parse_import(md);
        assert_eq!(out.tasks.len(), 3);
        assert_eq!(out.tasks[0].title, "foo");
        assert_eq!(out.tasks[1].title, "bar");
        assert_eq!(out.tasks[2].title, "baz");
    }

    #[test]
    fn realistic_user_todo_file() {
        let md = "# TODO\n\n## Corp Job\n- a\n- b\n\n## Okul\n- c\n";
        let out = parse_import(md);
        assert_eq!(out.name.as_deref(), Some("TODO"));
        assert_eq!(out.tasks.len(), 2);
        assert_eq!(out.tasks[0].title, "Corp Job");
        assert_eq!(out.tasks[0].children.len(), 2);
        assert_eq!(out.tasks[1].title, "Okul");
    }

    #[test]
    fn input_without_h1_yields_no_name() {
        let md = "- a\n- b\n";
        let out = parse_import(md);
        assert!(out.name.is_none());
        assert_eq!(out.tasks.len(), 2);
    }

    #[test]
    fn export_round_trip_preserves_completion_for_subtasks() {
        let mut parent = Task::default();
        parent.title = "p".into();
        parent.status = Status::NotStarted;
        let mut child_done = Task::default();
        child_done.title = "c1".into();
        child_done.status = Status::Completed;
        let mut child_todo = Task::default();
        child_todo.title = "c2".into();
        child_todo.status = Status::NotStarted;
        parent.sub_tasks = vec![child_done, child_todo];
        let md = parent.markdown();
        assert!(md.contains("- [ ] p"));
        assert!(md.contains("  - [x] c1"));
        assert!(md.contains("  - [ ] c2"));
    }
}
