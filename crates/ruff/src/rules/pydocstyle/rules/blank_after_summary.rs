use ruff_diagnostics::{AutofixKind, Availability, Diagnostic, Fix, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::newlines::StrExt;
use ruff_python_ast::types::Range;

use crate::checkers::ast::Checker;
use crate::docstrings::definition::Docstring;
use crate::message::Location;
use crate::registry::AsRule;

#[violation]
pub struct BlankLineAfterSummary {
    pub num_lines: usize,
}

fn fmt_blank_line_after_summary_autofix_msg(_: &BlankLineAfterSummary) -> String {
    "Insert single blank line".to_string()
}
impl Violation for BlankLineAfterSummary {
    const AUTOFIX: Option<AutofixKind> = Some(AutofixKind::new(Availability::Sometimes));

    #[derive_message_formats]
    fn message(&self) -> String {
        let BlankLineAfterSummary { num_lines } = self;
        if *num_lines == 0 {
            format!("1 blank line required between summary line and description")
        } else {
            format!(
                "1 blank line required between summary line and description (found {num_lines})"
            )
        }
    }

    fn autofix_title_formatter(&self) -> Option<fn(&Self) -> String> {
        let BlankLineAfterSummary { num_lines } = self;
        if *num_lines > 0 {
            return Some(fmt_blank_line_after_summary_autofix_msg);
        }
        None
    }
}

/// D205
pub fn blank_after_summary(checker: &mut Checker, docstring: &Docstring) {
    let body = docstring.body;

    // Find the "summary" line (defined as the first non-blank line).
    // The summary line may also be composed of multiple lines ending in
    // a backslash, in that case find the last of those lines.
    let mut summary_line = 1;
    for line in body.universal_newlines() {
        if line.trim().is_empty() || line.trim().ends_with('\\') {
            summary_line += 1;
        } else {
            break;
        }
    }
    let mut lines_count = 1;
    let mut blanks_count = 0;
    for line in body.trim().universal_newlines().skip(summary_line) {
        lines_count += 1;
        if line.trim().is_empty() {
            blanks_count += 1;
        } else {
            break;
        }
    }

    // This rule does not allow multi-line summaries (unless collapsed by ending the
    // line in a backslash).
    if lines_count > 1 && blanks_count != 1 {
        let mut diagnostic = Diagnostic::new(
            BlankLineAfterSummary {
                num_lines: blanks_count,
            },
            Range::from(docstring.expr),
        );
        if checker.patch(diagnostic.kind.rule()) {
            if blanks_count > 1 {
                // Insert one blank line after the summary (replacing any existing lines).
                diagnostic.amend(Fix::replacement(
                    checker.stylist.line_ending().to_string(),
                    Location::new(docstring.expr.location.row() + summary_line + 1, 0),
                    Location::new(
                        docstring.expr.location.row() + summary_line + 1 + blanks_count,
                        0,
                    ),
                ));
            }
        }
        checker.diagnostics.push(diagnostic);
    }
}
