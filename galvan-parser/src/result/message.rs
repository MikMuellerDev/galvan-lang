use std::borrow::Cow;
use std::ops::Range;

use annotate_snippets::display_list::FormatOptions;
pub use annotate_snippets::snippet::AnnotationType;
pub use annotate_snippets::snippet::SourceAnnotation;
pub use annotate_snippets::snippet::{Annotation, Slice, Snippet};

use crate::Source;

pub type Span = Range<usize>;

pub trait AsParserMessage {
    fn as_parser_message(&self, src: Source) -> ParserMessage<'_>;
}

pub type MessageType = AnnotationType;
pub struct ParserMessage<'a> {
    pub issue: Cow<'a, str>,
    // TODO: This should probably be its own data structure instead of just a string
    pub hint: Option<String>,
    pub msg_type: MessageType,
    pub src: Source,
    pub annotations: Vec<SourceAnnotation<'a>>,
}

impl<'a> ParserMessage<'a> {
    pub fn as_snippet(&'a self) -> Snippet<'a> {
        let ParserMessage {
            issue,
            hint,
            src,
            annotations,
            msg_type,
        } = self;

        Snippet {
            title: Some(Annotation {
                label: Some(issue),
                // TODO: compiler errors should have ids that provide explanation for them
                id: None,
                annotation_type: *msg_type,
            }),
            footer: hint
                .as_deref()
                .map(|h| Annotation {
                    id: None,
                    label: Some(h),
                    annotation_type: AnnotationType::Note,
                })
                .into_iter()
                .collect(),
            slices: vec![Slice {
                source: src.content(),
                line_start: 0,
                origin: src.origin(),
                annotations: annotations
                    .iter()
                    .map(|a| SourceAnnotation {
                        range: a.range,
                        label: a.label,
                        annotation_type: a.annotation_type,
                    })
                    .collect(),
                fold: false,
            }],
            opt: FormatOptions {
                color: true,
                ..Default::default()
            },
        }
    }
}
