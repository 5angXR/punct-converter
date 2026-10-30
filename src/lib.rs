use encoding_rs::GB18030;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedEncoding {
    Utf8 { bom: bool },
    Gb18030,
}

impl DetectedEncoding {
    pub fn label(&self) -> &'static str {
        match self {
            DetectedEncoding::Utf8 { .. } => "UTF-8",
            DetectedEncoding::Gb18030 => "GB18030",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    LikelyBinary,
}

/// Decodes raw bytes as UTF-8 (with or without BOM) or GB18030, rejecting binary inputs.
pub fn decode(bytes: &[u8]) -> Result<(String, DetectedEncoding), DecodeError> {
    if bytes.contains(&0) {
        return Err(DecodeError::LikelyBinary);
    }

    if let Some(stripped) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return match std::str::from_utf8(stripped) {
            Ok(s) => Ok((s.to_string(), DetectedEncoding::Utf8 { bom: true })),
            Err(_) => Err(DecodeError::LikelyBinary),
        };
    }

    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok((s.to_string(), DetectedEncoding::Utf8 { bom: false }));
    }

    // Use `decode_without_bom_handling` so WHATWG BOM sniffing does not reinterpret UTF-16 BOMs.
    let (cow, had_errors) = GB18030.decode_without_bom_handling(bytes);
    if had_errors {
        Err(DecodeError::LikelyBinary)
    } else {
        Ok((cow.into_owned(), DetectedEncoding::Gb18030))
    }
}

pub fn encode(text: &str, original: DetectedEncoding, force_utf8: bool) -> Vec<u8> {
    if force_utf8 {
        return text.as_bytes().to_vec();
    }
    match original {
        DetectedEncoding::Utf8 { bom } => {
            let mut out = Vec::with_capacity(text.len() + 3);
            if bom {
                out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            }
            out.extend_from_slice(text.as_bytes());
            out
        }
        DetectedEncoding::Gb18030 => {
            let (cow, _enc, _had_errors) = GB18030.encode(text);
            cow.into_owned()
        }
    }
}

fn map_char(c: char, convert_quotes: bool) -> Option<&'static str> {
    if !convert_quotes && matches!(c, '“' | '”' | '‘' | '’') {
        return None;
    }
    match c {
        '，' => Some(","),
        '。' => Some("."),
        '！' => Some("!"),
        '？' => Some("?"),
        '：' => Some(":"),
        '；' => Some(";"),
        '、' => Some(","),
        '（' => Some("("),
        '）' => Some(")"),
        '“' | '”' => Some("\""),
        '‘' | '’' => Some("'"),
        '【' | '〔' => Some("["),
        '】' | '〕' => Some("]"),
        '《' | '〈' => Some("<"),
        '》' | '〉' => Some(">"),
        '～' => Some("~"),
        '％' => Some("%"),
        '＃' => Some("#"),
        '＆' => Some("&"),
        '＊' => Some("*"),
        '＋' => Some("+"),
        '－' => Some("-"),
        '＝' => Some("="),
        '＠' => Some("@"),
        '｜' => Some("|"),
        '＼' => Some("\\"),
        '／' => Some("/"),
        '＿' => Some("_"),
        '｀' => Some("`"),
        '＾' => Some("^"),
        '｛' => Some("{"),
        '｝' => Some("}"),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertResult {
    pub text: String,
    pub replacements: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct ConvertOptions {
    pub space_after_terminal_punct: bool,
    pub convert_quotes: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        ConvertOptions {
            space_after_terminal_punct: true,
            convert_quotes: true,
        }
    }
}

fn is_terminal_punct(c: char) -> bool {
    matches!(c, '，' | '。' | '！' | '？' | '：' | '；' | '、')
}

pub fn convert_text(input: &str) -> ConvertResult {
    convert_text_with_options(input, &ConvertOptions::default())
}

pub fn convert_text_with_options(input: &str, opts: &ConvertOptions) -> ConvertResult {
    let mut out = String::with_capacity(input.len());
    let mut replacements = 0usize;
    let mut chars = input.chars().peekable();
    let mut prev_char: Option<char> = None;

    while let Some(c) = chars.next() {
        // Merge Chinese double ellipsis (`……`) or single (`…`) into `...`.
        if matches!(c, '…' | '⋯') {
            if matches!(chars.peek(), Some('…' | '⋯')) {
                chars.next();
            }
            out.push_str("...");
            replacements += 1;
            prev_char = Some('.');
            continue;
        }

        // Merge Chinese double em-dash (`——`) or single (`—`) into `--`.
        if matches!(c, '—' | '―') {
            if matches!(chars.peek(), Some('—' | '―')) {
                chars.next();
            }
            out.push_str("--");
            replacements += 1;
            prev_char = Some('-');
            continue;
        }

        match map_char(c, opts.convert_quotes) {
            Some(rep) => {
                out.push_str(rep);
                replacements += 1;

                if opts.space_after_terminal_punct && is_terminal_punct(c) {
                    // Keep numeric literals intact (e.g. `3.14`, `12:30`, `1,000`).
                    let is_between_digits = prev_char.is_some_and(|p| p.is_ascii_digit())
                        && chars.peek().is_some_and(|n| n.is_ascii_digit())
                        && matches!(c, '。' | '：' | '，');

                    if !is_between_digits && chars.peek().is_some_and(|n| n.is_alphanumeric()) {
                        out.push(' ');
                    }
                }
                prev_char = rep.chars().last();
            }
            None => {
                out.push(c);
                prev_char = Some(c);
            }
        }
    }

    ConvertResult {
        text: out,
        replacements,
    }
}
