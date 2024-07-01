use std::fmt::Write;

#[cfg(test)]
mod tests;

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Source {
    pub other: String,
    pub predicate: String,
    pub constraint_line: Option<usize>,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ShowOutput {
    All,
    Predicate,
    #[default]
    Constraint,
    ConstraintOnly,
}

pub fn show_code(source: &Option<Source>, show: ShowOutput) -> String {
    match source {
        Some(source) => match show {
            ShowOutput::All => format!(
                "{}\n{}",
                source.other,
                format_predicate(&source.predicate, &source.constraint_line)
            ),
            ShowOutput::Predicate => format_predicate(&source.predicate, &source.constraint_line),
            ShowOutput::Constraint => format_constraint(&source.predicate, &source.constraint_line),
            ShowOutput::ConstraintOnly => {
                constraint_only(&source.predicate, &source.constraint_line)
            }
        },
        None => "No source code available.".to_string(),
    }
}

impl Source {
    pub fn with_predicate(self, predicate: impl Into<String>) -> Self {
        Source {
            predicate: predicate.into(),
            ..self
        }
    }

    pub fn with_predicate_find_line(
        self,
        predicate: impl Into<String>,
        constraint_num: usize,
    ) -> Self {
        let predicate = predicate.into();
        let mut count = 0;
        let constraint_line = predicate.lines().position(|line| {
            if line.trim().starts_with("constraint ") {
                let found = count == constraint_num;
                count += 1;
                found
            } else {
                false
            }
        });
        Source {
            predicate,
            constraint_line,
            ..self
        }
    }

    pub fn with_constraint_line_number(self, constraint_line: usize) -> Self {
        Source {
            constraint_line: Some(constraint_line),
            ..self
        }
    }

    pub fn with_other_code(self, other: impl Into<String>) -> Self {
        Source {
            other: other.into(),
            ..self
        }
    }
}

fn format_predicate(predicate: &str, constraint_line: &Option<usize>) -> String {
    match constraint_line {
        Some(line_num) => predicate.lines().enumerate().fold(
            String::with_capacity(predicate.len()),
            |mut s, (i, line)| {
                if i == *line_num {
                    let _ = writeln!(s, "{}", dialoguer::console::style(line).cyan());
                } else {
                    let _ = writeln!(s, "{}", line);
                }
                s
            },
        ),
        None => predicate.to_string(),
    }
}

fn format_constraint(predicate: &str, constraint_line: &Option<usize>) -> String {
    match constraint_line {
        Some(line_num) => predicate.lines().enumerate().fold(
            String::with_capacity(predicate.len()),
            |mut s, (i, line)| {
                if i == *line_num {
                    let _ = writeln!(s, "{}", line);
                } else if line.trim().starts_with("constraint ") {
                } else {
                    let _ = writeln!(s, "{}", line);
                }
                s
            },
        ),
        None => predicate.to_string(),
    }
}

fn constraint_only(predicate: &str, constraint_line: &Option<usize>) -> String {
    match constraint_line {
        Some(line_num) => match predicate.lines().nth(*line_num) {
            Some(line) => line.trim().to_string(),
            None => predicate.to_string(),
        },
        None => predicate.to_string(),
    }
}

impl From<Option<&str>> for ShowOutput {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some("a") | Some("all") => ShowOutput::All,
            Some("p") | Some("predicate") => ShowOutput::Predicate,
            Some("c") | Some("constraint") => ShowOutput::Constraint,
            Some("co") | Some("constraint only") => ShowOutput::ConstraintOnly,
            _ => ShowOutput::default(),
        }
    }
}
