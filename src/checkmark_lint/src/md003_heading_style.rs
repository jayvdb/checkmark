use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, MarkDownFile};
use markdown::{
    mdast::{self},
    to_mdast, ParseOptions,
};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD003")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md003.md")
        .is_fmt_fixable(true)
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeadingStyle {
    Consistent,
    Atx,
    SetExt,
}

impl HeadingStyle {
    pub fn as_str(&self) -> &str {
        match self {
            HeadingStyle::Consistent => "consistent",
            HeadingStyle::Atx => "ATX",
            HeadingStyle::SetExt => "SetExt",
        }
    }
}

pub fn md003_heading_style(file: &MarkDownFile, style: &HeadingStyle) -> Vec<Violation> {
    log::debug!("[MD003] File: {:#?}, style: {:#?}", &file.path, &style);

    let ast = to_mdast(&file.content, &ParseOptions::gfm()).unwrap();

    // Get all headings
    let mut headings: Vec<&mdast::Heading> = vec![];
    for_each(&ast, |node| {
        if let mdast::Node::Heading(h) = node {
            headings.push(h);
        }
    });
    log::debug!("[MD003] Headings: {:#?}", &headings);

    let get_heading_style = |h: &mdast::Heading, source: &str| -> HeadingStyle {
        let offset_start = h.position.as_ref().unwrap().start.offset;
        let offset_end = h.position.as_ref().unwrap().end.offset;
        let text = source.get(offset_start..offset_end).unwrap_or("");
        if text.starts_with("#") {
            HeadingStyle::Atx
        } else {
            HeadingStyle::SetExt
        }
    };

    let preferred_style = match style {
        HeadingStyle::Consistent => {
            if let Some(h) = headings.first() {
                get_heading_style(&h, &file.content)
            } else {
                HeadingStyle::Atx
            }
        }
        HeadingStyle::Atx => HeadingStyle::Atx,
        HeadingStyle::SetExt => HeadingStyle::SetExt,
    };
    log::debug!(
        "[MD003] Document should have heading style: {:#?}",
        &preferred_style
    );

    let input_style_string = match style {
        HeadingStyle::Consistent => "consistent",
        HeadingStyle::Atx => "atx",
        HeadingStyle::SetExt => "setext",
    };

    headings
        .iter()
        .filter(|h| get_heading_style(&h, &file.content).ne(&preferred_style))
        .map(|h| {
            let mut violation = violation_builder();
            if style.eq(&HeadingStyle::Consistent) {
                violation = violation.message(&format!(
                    "Inconsistent headings style. First heading in this file is {:#?}, but this one is {:#?}",
                    preferred_style.as_str(),
                    get_heading_style(&h, &file.content).as_str()
                ))
            } else {
                violation = violation.message(&format!(
                    "Wrong heading style. Expected {:#?}, got {:#?}",
                    input_style_string,
                    get_heading_style(&h, &file.content).as_str()
                ))
            }
            violation
                .push_fix(&format!("Change heading style to {:#?}", preferred_style.as_str()))
                .push_fix(&format!("Alternatively, you can enforce specific heading style via either \"headings\" option from the \"[style]\" section in config file or via \"--style-headings\" CLI option"))
                .push_fix("See Markdown headings reference: https://www.markdownguide.org/basic-syntax/#headings")
                .position(&h.position)
                .build()
        })
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn md003() {
        let mut file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1
        
H2
-----"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Inconsistent headings style. First heading in this file is \"ATX\", but this one is \"SetExt\"")
                .position(&Some(markdown::unist::Position::new(3, 1, 14, 4, 6, 22)))
                .build()],
            md003_heading_style(&file, &HeadingStyle::Consistent),
        );

        file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1".to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Wrong heading style. Expected \"setext\", got \"ATX\"")
                .position(&Some(markdown::unist::Position::new(1, 1, 0, 1, 5, 4)))
                .build()],
            md003_heading_style(&file, &HeadingStyle::SetExt),
        );

        file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "H1
==========="
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Wrong heading style. Expected \"atx\", got \"SetExt\"")
                .position(&Some(markdown::unist::Position::new(1, 1, 0, 2, 12, 14)))
                .build()],
            md003_heading_style(&file, &HeadingStyle::Atx),
        );
    }
}