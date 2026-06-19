use arcaea_viewer_core::{ArcColor, ArcCurve};

use crate::{Diagnostic, Span};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Spanned<T> {
    pub(crate) value: T,
    pub(crate) span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SyntaxEvent {
    TimingGroupDefinition {
        timing_group_id: u32,
        properties: Vec<Spanned<String>>,
    },
    Tap {
        timing_group_id: u32,
        time: Spanned<i64>,
        lane: Spanned<u8>,
    },
    Hold {
        timing_group_id: u32,
        start_time: Spanned<i64>,
        end_time: Spanned<i64>,
        lane: Spanned<u8>,
    },
    Timing {
        timing_group_id: u32,
        time: Spanned<i64>,
        tempo_milli_bpm: Spanned<u32>,
        beats_per_measure: Spanned<u16>,
    },
    Arc {
        timing_group_id: u32,
        start_time: Spanned<i64>,
        end_time: Spanned<i64>,
        start_x: Spanned<f32>,
        end_x: Spanned<f32>,
        curve: Spanned<ArcCurve>,
        start_y: Spanned<f32>,
        end_y: Spanned<f32>,
        color: Spanned<ArcColor>,
        is_trace: Spanned<bool>,
        arc_taps: Vec<Spanned<i64>>,
    },
}

#[derive(Debug)]
pub(crate) struct SyntaxParser<'a> {
    source: &'a str,
}

impl<'a> SyntaxParser<'a> {
    pub(crate) const fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub(crate) fn parse(&self) -> (Vec<SyntaxEvent>, Vec<Diagnostic>) {
        let mut events = Vec::new();
        let mut diagnostics = Vec::new();
        let mut line_start = 0_usize;
        let mut current_group_id = 0_u32;
        let mut open_group_span: Option<Span> = None;
        let mut next_group_id = 1_u32;

        for raw_line in self.source.split_inclusive('\n') {
            let line = raw_line.strip_suffix('\n').unwrap_or(raw_line);
            let line = line.strip_suffix('\r').unwrap_or(line);
            if let Some((text, span_start)) = content_text(line, line_start) {
                if text.starts_with("timinggroup") {
                    if open_group_span.is_some() {
                        diagnostics.push(Diagnostic::syntax(
                            "nested timinggroup blocks are not supported",
                            Span::new(span_start, span_start + text.len()),
                            None,
                            Some("close the current timinggroup before starting another one"),
                        ));
                    } else if let Some(properties) =
                        parse_timing_group_open(text, span_start, &mut diagnostics)
                    {
                        let timing_group_id = next_group_id;
                        next_group_id += 1;
                        current_group_id = timing_group_id;
                        open_group_span = Some(Span::new(span_start, span_start + text.len()));
                        events.push(SyntaxEvent::TimingGroupDefinition {
                            timing_group_id,
                            properties,
                        });
                    }
                } else if text == "};" {
                    if open_group_span.is_some() {
                        current_group_id = 0;
                        open_group_span = None;
                    } else {
                        diagnostics.push(Diagnostic::syntax(
                            "unexpected timinggroup close",
                            Span::new(span_start, span_start + text.len()),
                            None,
                            Some("remove `};` or add a matching timinggroup opening line"),
                        ));
                    }
                } else if let Some(event) =
                    self.parse_line(text, span_start, current_group_id, &mut diagnostics)
                {
                    events.push(event);
                }
            }
            line_start += raw_line.len();
        }

        if let Some(span) = open_group_span {
            diagnostics.push(Diagnostic::syntax(
                "unterminated timinggroup block",
                span,
                None,
                Some("close the timinggroup block with `};`"),
            ));
        }

        (events, diagnostics)
    }

    fn parse_line(
        &self,
        text: &str,
        span_start: usize,
        timing_group_id: u32,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<SyntaxEvent> {
        if text.starts_with('(') {
            return self.parse_tap(text, span_start, timing_group_id, diagnostics);
        }

        let Some(name_end) = text.find('(') else {
            diagnostics.push(Diagnostic::syntax(
                "expected AFF event statement",
                Span::new(span_start, span_start + text.len()),
                Some(
                    "event must look like timing(...);, hold(...);, arc(...);, or (time,lane);"
                        .into(),
                ),
                None,
            ));
            return None;
        };

        let name = &text[..name_end];
        match name {
            "timing" => self.parse_timing(text, span_start, timing_group_id, diagnostics),
            "hold" => self.parse_hold(text, span_start, timing_group_id, diagnostics),
            "arc" => self.parse_arc(text, span_start, timing_group_id, diagnostics),
            "arctap" => {
                diagnostics.push(Diagnostic::unsupported(
                    "arc tap without parent arc",
                    Span::new(span_start, span_start + name_end),
                    "arctap must appear inside an arc extension block".into(),
                ));
                None
            }
            unsupported => {
                diagnostics.push(Diagnostic::unsupported(
                    "unsupported AFF event",
                    Span::new(span_start, span_start + name_end),
                    format!("event `{unsupported}` is recognized as a named AFF-style event but is not implemented here"),
                ));
                None
            }
        }
    }

    fn parse_tap(
        &self,
        text: &str,
        base: usize,
        timing_group_id: u32,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<SyntaxEvent> {
        let fields = parse_call_fields(text, base, None, diagnostics)?;
        if fields.len() != 2 {
            diagnostics.push(Diagnostic::syntax(
                "tap note expects 2 fields",
                Span::new(base, base + text.len()),
                Some(format!("got {} fields", fields.len())),
                Some("expected (time,lane);"),
            ));
            return None;
        }

        Some(SyntaxEvent::Tap {
            timing_group_id,
            time: parse_i64(fields[0], diagnostics)?,
            lane: parse_u8(fields[1], diagnostics)?,
        })
    }

    fn parse_hold(
        &self,
        text: &str,
        base: usize,
        timing_group_id: u32,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<SyntaxEvent> {
        let fields = parse_call_fields(text, base, Some("hold"), diagnostics)?;
        if fields.len() != 3 {
            diagnostics.push(Diagnostic::syntax(
                "hold note expects 3 fields",
                Span::new(base, base + text.len()),
                Some(format!("got {} fields", fields.len())),
                Some("expected hold(start,end,lane);"),
            ));
            return None;
        }

        Some(SyntaxEvent::Hold {
            timing_group_id,
            start_time: parse_i64(fields[0], diagnostics)?,
            end_time: parse_i64(fields[1], diagnostics)?,
            lane: parse_u8(fields[2], diagnostics)?,
        })
    }

    fn parse_timing(
        &self,
        text: &str,
        base: usize,
        timing_group_id: u32,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<SyntaxEvent> {
        let fields = parse_call_fields(text, base, Some("timing"), diagnostics)?;
        if fields.len() != 3 {
            diagnostics.push(Diagnostic::syntax(
                "timing event expects 3 fields",
                Span::new(base, base + text.len()),
                Some(format!("got {} fields", fields.len())),
                Some("expected timing(time,bpm,beats_per_measure);"),
            ));
            return None;
        }

        Some(SyntaxEvent::Timing {
            timing_group_id,
            time: parse_i64(fields[0], diagnostics)?,
            tempo_milli_bpm: parse_milli_bpm(fields[1], diagnostics)?,
            beats_per_measure: parse_u16(fields[2], diagnostics)?,
        })
    }

    fn parse_arc(
        &self,
        text: &str,
        base: usize,
        timing_group_id: u32,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<SyntaxEvent> {
        let (call_text, arc_taps) = if let Some(open) = text.find('[') {
            let close = text.rfind(']')?;
            if close + 2 != text.len() || !text.ends_with("];") {
                diagnostics.push(Diagnostic::syntax(
                    "arc extension must end with `];`",
                    Span::new(base + open, base + text.len()),
                    None,
                    Some("expected arc(...)[arctap(time),...];"),
                ));
                return None;
            }
            let prefix = text[..open].trim_end();
            let mut owned = prefix.to_owned();
            owned.push(';');
            let taps = parse_arc_taps(&text[open + 1..close], base + open + 1, diagnostics)?;
            (owned, taps)
        } else if text.contains(']') {
            diagnostics.push(Diagnostic::syntax(
                "unexpected arc extension close",
                Span::new(base, base + text.len()),
                None,
                None,
            ));
            return None;
        } else {
            (text.to_owned(), Vec::new())
        };

        let fields = parse_call_fields(&call_text, base, Some("arc"), diagnostics)?;
        if fields.len() != 10 {
            diagnostics.push(Diagnostic::syntax(
                "arc note expects 10 fields",
                Span::new(base, base + text.len()),
                Some(format!("got {} fields", fields.len())),
                Some("expected arc(start,end,x1,x2,curve,y1,y2,color,fx,is_trace);"),
            ));
            return None;
        }

        Some(SyntaxEvent::Arc {
            timing_group_id,
            start_time: parse_i64(fields[0], diagnostics)?,
            end_time: parse_i64(fields[1], diagnostics)?,
            start_x: parse_f32(fields[2], diagnostics)?,
            end_x: parse_f32(fields[3], diagnostics)?,
            curve: parse_curve(fields[4], diagnostics)?,
            start_y: parse_f32(fields[5], diagnostics)?,
            end_y: parse_f32(fields[6], diagnostics)?,
            color: parse_color(fields[7], diagnostics)?,
            is_trace: parse_bool(fields[9], diagnostics)?,
            arc_taps,
        })
    }
}

fn content_text(line: &str, line_start: usize) -> Option<(&str, usize)> {
    let content_len = line.find("//").unwrap_or(line.len());
    let content = &line[..content_len];
    let leading = content.len() - content.trim_start().len();
    let trailing = content.trim_end().len();

    if leading >= trailing {
        None
    } else {
        Some((&content[leading..trailing], line_start + leading))
    }
}

fn parse_timing_group_open(
    text: &str,
    base: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Vec<Spanned<String>>> {
    if !text.ends_with('{') {
        diagnostics.push(Diagnostic::syntax(
            "timinggroup block must end with `{`",
            Span::new(base, base + text.len()),
            None,
            Some("expected timinggroup(...){"),
        ));
        return None;
    }

    let head = text[..text.len() - 1].trim_end();
    if head == "timinggroup" {
        return Some(Vec::new());
    }
    if !head.starts_with("timinggroup(") || !head.ends_with(')') {
        diagnostics.push(Diagnostic::syntax(
            "expected timinggroup properties",
            Span::new(base, base + text.len()),
            None,
            Some("expected timinggroup(noinput,noclip){ or timinggroup(){"),
        ));
        return None;
    }

    let inner_start = "timinggroup(".len();
    let inner = &head[inner_start..head.len() - 1];
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut properties = Vec::new();
    let mut field_start = 0_usize;
    for (index, byte) in inner.bytes().enumerate() {
        if byte == b',' {
            properties.push(trim_string_field(
                inner,
                inner_start,
                field_start,
                index,
                base,
            ));
            field_start = index + 1;
        }
    }
    properties.push(trim_string_field(
        inner,
        inner_start,
        field_start,
        inner.len(),
        base,
    ));

    if properties.iter().any(|property| property.value.is_empty()) {
        diagnostics.push(Diagnostic::syntax(
            "empty timinggroup property",
            Span::new(base, base + text.len()),
            None,
            Some("remove the empty property or provide a supported property name"),
        ));
        return None;
    }
    Some(properties)
}

fn parse_arc_taps(
    inner: &str,
    base: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Vec<Spanned<i64>>> {
    let mut taps = Vec::new();
    let mut cursor = 0_usize;
    while cursor < inner.len() {
        while inner
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if cursor >= inner.len() {
            break;
        }
        if !inner[cursor..].starts_with("arctap(") {
            diagnostics.push(Diagnostic::unsupported(
                "unsupported arc extension",
                Span::new(base + cursor, base + inner.len()),
                "only arctap(time) is supported in arc extension blocks".into(),
            ));
            return None;
        }
        let value_start = cursor + "arctap(".len();
        let Some(close_offset) = inner[value_start..].find(')') else {
            diagnostics.push(Diagnostic::syntax(
                "unterminated arctap",
                Span::new(base + cursor, base + inner.len()),
                None,
                Some("expected arctap(time)"),
            ));
            return None;
        };
        let value_end = value_start + close_offset;
        let field = trim_field(inner, 0, value_start, value_end, base);
        taps.push(parse_i64(field, diagnostics)?);
        cursor = value_end + 1;
        while inner
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if cursor < inner.len() {
            if inner.as_bytes().get(cursor) == Some(&b',') {
                cursor += 1;
            } else {
                diagnostics.push(Diagnostic::syntax(
                    "expected comma between arctaps",
                    Span::new(base + cursor, base + cursor + 1),
                    None,
                    Some("expected arctap(a),arctap(b)"),
                ));
                return None;
            }
        }
    }
    Some(taps)
}

fn parse_call_fields<'a>(
    text: &'a str,
    base: usize,
    name: Option<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Vec<Field<'a>>> {
    let open = if let Some(name) = name {
        let prefix = format!("{name}(");
        if !text.starts_with(&prefix) {
            diagnostics.push(Diagnostic::syntax(
                format!("expected `{name}(`"),
                Span::new(base, base + text.len()),
                None,
                None,
            ));
            return None;
        }
        name.len()
    } else {
        0
    };

    if !text.ends_with(';') {
        diagnostics.push(Diagnostic::syntax(
            "statement must end with semicolon",
            Span::new(base + text.len().saturating_sub(1), base + text.len()),
            None,
            Some("add `;` at the end of the AFF event"),
        ));
        return None;
    }

    let close = text.len().saturating_sub(2);
    if text.as_bytes().get(open) != Some(&b'(') || text.as_bytes().get(close) != Some(&b')') {
        diagnostics.push(Diagnostic::syntax(
            "expected parenthesized event fields",
            Span::new(base, base + text.len()),
            None,
            None,
        ));
        return None;
    }

    let inner_start = open + 1;
    let inner = &text[inner_start..close];
    let mut fields = Vec::new();
    let mut field_start = 0_usize;

    for (index, byte) in inner.bytes().enumerate() {
        if byte == b',' {
            fields.push(trim_field(inner, inner_start, field_start, index, base));
            field_start = index + 1;
        }
    }
    fields.push(trim_field(
        inner,
        inner_start,
        field_start,
        inner.len(),
        base,
    ));

    if fields.iter().any(|field| field.text.is_empty()) {
        let span = fields
            .iter()
            .find(|field| field.text.is_empty())
            .map_or(Span::new(base, base + text.len()), |field| field.span);
        diagnostics.push(Diagnostic::syntax(
            "empty event field",
            span,
            None,
            Some("provide a value between each comma"),
        ));
        return None;
    }

    Some(fields)
}

#[derive(Debug, Clone, Copy)]
struct Field<'a> {
    text: &'a str,
    span: Span,
}

fn trim_field<'a>(
    inner: &'a str,
    inner_start: usize,
    raw_start: usize,
    raw_end: usize,
    base: usize,
) -> Field<'a> {
    let raw = &inner[raw_start..raw_end];
    let leading = raw.len() - raw.trim_start().len();
    let trailing = raw.trim_end().len();
    let start = raw_start + leading;
    let end = raw_start + trailing;

    Field {
        text: &inner[start..end],
        span: Span::new(base + inner_start + start, base + inner_start + end),
    }
}

fn trim_string_field(
    inner: &str,
    inner_start: usize,
    raw_start: usize,
    raw_end: usize,
    base: usize,
) -> Spanned<String> {
    let field = trim_field(inner, inner_start, raw_start, raw_end, base);
    Spanned {
        value: field.text.to_owned(),
        span: field.span,
    }
}

fn parse_i64(field: Field<'_>, diagnostics: &mut Vec<Diagnostic>) -> Option<Spanned<i64>> {
    match field.text.parse::<i64>() {
        Ok(value) => Some(Spanned {
            value,
            span: field.span,
        }),
        Err(error) => {
            diagnostics.push(Diagnostic::lexical(
                "invalid integer",
                field.span,
                Some(error.to_string()),
                Some("expected a base-10 integer"),
            ));
            None
        }
    }
}

fn parse_u8(field: Field<'_>, diagnostics: &mut Vec<Diagnostic>) -> Option<Spanned<u8>> {
    match field.text.parse::<u8>() {
        Ok(value) => Some(Spanned {
            value,
            span: field.span,
        }),
        Err(error) => {
            diagnostics.push(Diagnostic::lexical(
                "invalid unsigned byte integer",
                field.span,
                Some(error.to_string()),
                Some("expected an integer in 0..=255"),
            ));
            None
        }
    }
}

fn parse_u16(field: Field<'_>, diagnostics: &mut Vec<Diagnostic>) -> Option<Spanned<u16>> {
    match field.text.parse::<u16>() {
        Ok(value) => Some(Spanned {
            value,
            span: field.span,
        }),
        Err(error) => {
            diagnostics.push(Diagnostic::lexical(
                "invalid unsigned integer",
                field.span,
                Some(error.to_string()),
                Some("expected an integer in 0..=65535"),
            ));
            None
        }
    }
}

fn parse_f32(field: Field<'_>, diagnostics: &mut Vec<Diagnostic>) -> Option<Spanned<f32>> {
    match field.text.parse::<f32>() {
        Ok(value) => Some(Spanned {
            value,
            span: field.span,
        }),
        Err(error) => {
            diagnostics.push(Diagnostic::lexical(
                "invalid decimal number",
                field.span,
                Some(error.to_string()),
                Some("expected a finite decimal number"),
            ));
            None
        }
    }
}

fn parse_milli_bpm(field: Field<'_>, diagnostics: &mut Vec<Diagnostic>) -> Option<Spanned<u32>> {
    let text = field.text;
    if text.starts_with('-') {
        diagnostics.push(Diagnostic::lexical(
            "invalid BPM",
            field.span,
            Some("BPM cannot be negative".into()),
            Some("expected a positive decimal BPM"),
        ));
        return None;
    }

    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.len() > 3
    {
        diagnostics.push(Diagnostic::lexical(
            "invalid BPM",
            field.span,
            Some("BPM supports up to three decimal places".into()),
            Some("examples: 120, 120.5, 120.000"),
        ));
        return None;
    }

    let whole = match whole.parse::<u32>() {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(Diagnostic::lexical(
                "invalid BPM",
                field.span,
                Some(error.to_string()),
                Some("expected BPM within u32 milli-BPM range"),
            ));
            return None;
        }
    };
    let mut milli = match whole.checked_mul(1_000) {
        Some(value) => value,
        None => {
            diagnostics.push(Diagnostic::lexical(
                "invalid BPM",
                field.span,
                Some("BPM is too large".into()),
                Some("expected BPM within u32 milli-BPM range"),
            ));
            return None;
        }
    };
    let mut fraction_text = fraction.to_owned();
    while fraction_text.len() < 3 {
        fraction_text.push('0');
    }
    if !fraction_text.is_empty() {
        let fraction = match fraction_text.parse::<u32>() {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(Diagnostic::lexical(
                    "invalid BPM",
                    field.span,
                    Some(error.to_string()),
                    Some("expected decimal BPM"),
                ));
                return None;
            }
        };
        milli = match milli.checked_add(fraction) {
            Some(value) => value,
            None => {
                diagnostics.push(Diagnostic::lexical(
                    "invalid BPM",
                    field.span,
                    Some("BPM is too large".into()),
                    Some("expected BPM within u32 milli-BPM range"),
                ));
                return None;
            }
        };
    }

    Some(Spanned {
        value: milli,
        span: field.span,
    })
}

fn parse_curve(field: Field<'_>, diagnostics: &mut Vec<Diagnostic>) -> Option<Spanned<ArcCurve>> {
    let value = match field.text {
        "s" => ArcCurve::Straight,
        "b" => ArcCurve::Bezier,
        "si" => ArcCurve::SineIn,
        "so" => ArcCurve::SineOut,
        "sisi" => ArcCurve::SineInOut,
        "soso" => ArcCurve::SineOutIn,
        other => {
            diagnostics.push(Diagnostic::syntax(
                "unsupported arc curve token",
                field.span,
                Some(format!("got `{other}`")),
                Some("expected one of s, b, si, so, sisi, soso"),
            ));
            return None;
        }
    };

    Some(Spanned {
        value,
        span: field.span,
    })
}

fn parse_color(field: Field<'_>, diagnostics: &mut Vec<Diagnostic>) -> Option<Spanned<ArcColor>> {
    let value = match field.text {
        "0" => ArcColor::Blue,
        "1" => ArcColor::Red,
        "2" => ArcColor::Green,
        other => {
            diagnostics.push(Diagnostic::syntax(
                "unsupported arc color",
                field.span,
                Some(format!("got `{other}`")),
                Some("expected 0 for blue, 1 for red, or 2 for green"),
            ));
            return None;
        }
    };

    Some(Spanned {
        value,
        span: field.span,
    })
}

fn parse_bool(field: Field<'_>, diagnostics: &mut Vec<Diagnostic>) -> Option<Spanned<bool>> {
    let value = match field.text {
        "true" => true,
        "false" => false,
        other => {
            diagnostics.push(Diagnostic::syntax(
                "invalid boolean",
                field.span,
                Some(format!("got `{other}`")),
                Some("expected true or false"),
            ));
            return None;
        }
    };

    Some(Spanned {
        value,
        span: field.span,
    })
}
