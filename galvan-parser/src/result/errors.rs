use crate::{AnnotationType, AsParserMessage, ParserMessage, Source, SourceAnnotation, Span};

pub struct TokenError {
    pub msg: String,
    pub span: Span,
    pub annotation: String,
}

impl AsParserMessage for TokenError {
    fn as_parser_message(&self, src: Source) -> ParserMessage<'_> {
        let TokenError {
            msg,
            span,
            annotation,
        } = self;

        ParserMessage {
            issue: msg.into(),
            hint: None,
            msg_type: AnnotationType::Error,
            src,
            // TODO: Create annotations
            annotations: vec![SourceAnnotation {
                label: annotation,
                range: (span.start, span.end),
                annotation_type: AnnotationType::Error,
            }],
        }
    }
}
