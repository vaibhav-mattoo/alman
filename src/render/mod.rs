use crate::registry::Definition;
use crate::template::{CommandTemplate, TemplatePart};

pub trait ShellRenderer: Send + Sync {
    fn slot_ref(&self, n: u32) -> String;
    fn quote_literal(&self, tok: &str) -> String;
    fn render_alias(&self, name: &str, command: &str) -> String;
    fn render_function(&self, name: &str, body: &str) -> String;

    fn render_definition(&self, d: &Definition) -> String {
        use crate::registry::DefinitionKind;
        match d.kind {
            DefinitionKind::Alias => {
                // zero-slot or trailing-single-slot → alias.
                // The command text is built raw here; `render_alias` quotes it once.
                let command = self.render_alias_command(&d.template);
                self.render_alias(&d.name, &command)
            }
            DefinitionKind::Function => {
                let body = self.render_template_body(&d.template);
                self.render_function(&d.name, &body)
            }
        }
    }

    /// Build the raw (unquoted) command text for an alias body. Slots are kept
    /// as raw positional refs so a trailing-single-slot alias still works when
    /// the shell appends arguments after the alias expansion.
    fn render_alias_command(&self, t: &CommandTemplate) -> String {
        t.parts
            .iter()
            .enumerate()
            .map(|(i, part)| {
                let s = match part {
                    TemplatePart::Literal(s) => s.clone(),
                    TemplatePart::Slot(n) => self.slot_ref(*n),
                };
                if i == 0 {
                    s
                } else {
                    format!(" {}", s)
                }
            })
            .collect()
    }

    fn render_template_body(&self, t: &CommandTemplate) -> String {
        t.parts
            .iter()
            .enumerate()
            .map(|(i, part)| {
                let s = match part {
                    TemplatePart::Literal(s) => self.quote_literal(s),
                    TemplatePart::Slot(n) => self.slot_ref(*n),
                };
                if i == 0 {
                    s
                } else {
                    format!(" {}", s)
                }
            })
            .collect()
    }
}

pub struct PosixRenderer;

impl ShellRenderer for PosixRenderer {
    fn slot_ref(&self, n: u32) -> String {
        format!("\"${}\"", n)
    }
    fn quote_literal(&self, tok: &str) -> String {
        // single-quote with embedded ' escaped as '\''
        format!("'{}'", tok.replace('\'', r"'\''"))
    }
    fn render_alias(&self, name: &str, command: &str) -> String {
        format!("alias {}={}", name, self.quote_literal(command))
    }
    fn render_function(&self, name: &str, body: &str) -> String {
        format!("{}() {{ {}; }}", name, body)
    }
}

pub struct FishRenderer;

impl ShellRenderer for FishRenderer {
    fn slot_ref(&self, n: u32) -> String {
        format!("$argv[{}]", n)
    }
    fn quote_literal(&self, tok: &str) -> String {
        // fish uses double-quotes; escape " and \
        format!("\"{}\"", tok.replace('\\', "\\\\").replace('"', "\\\""))
    }
    fn render_alias(&self, name: &str, command: &str) -> String {
        format!("alias {} {}", name, self.quote_literal(command))
    }
    fn render_function(&self, name: &str, body: &str) -> String {
        format!("function {}\n    {}\nend", name, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{Definition, DefinitionKind};

    fn interior_def() -> Definition {
        Definition {
            name: "dex".into(),
            kind: DefinitionKind::Function,
            template: CommandTemplate {
                parts: vec![
                    TemplatePart::Literal("docker".into()),
                    TemplatePart::Literal("exec".into()),
                    TemplatePart::Slot(1),
                    TemplatePart::Literal("bash".into()),
                ],
            },
        }
    }

    fn zero_def() -> Definition {
        Definition {
            name: "gs".into(),
            kind: DefinitionKind::Alias,
            template: CommandTemplate {
                parts: vec![TemplatePart::Literal("git status".into())],
            },
        }
    }

    #[test]
    fn posix_interior_slot_renders_function() {
        let out = PosixRenderer.render_definition(&interior_def());
        assert!(out.starts_with("dex() {"), "got {out}");
        assert!(out.contains("\"$1\""), "slot ref missing: {out}");
        assert!(out.ends_with("; }"), "got {out}");
    }

    #[test]
    fn posix_zero_slot_renders_alias() {
        let out = PosixRenderer.render_definition(&zero_def());
        assert_eq!(out, "alias gs='git status'");
    }

    #[test]
    fn fish_uses_argv_and_function_end() {
        let out = FishRenderer.render_definition(&interior_def());
        assert!(out.contains("$argv[1]"), "got {out}");
        assert!(out.starts_with("function dex"), "got {out}");
        assert!(out.ends_with("end"), "got {out}");
    }
}
