use encoding_rs::GB18030;
use punct_converter::{
    ConvertOptions, DecodeError, DetectedEncoding, convert_text, convert_text_with_options, decode,
    encode,
};

#[test]
fn test_fixture_file_conversion() {
    let input = include_str!("fixtures/input.txt");
    let expected = include_str!("fixtures/expected.txt");

    let result = convert_text(input);
    assert_eq!(result.text, expected);
}

#[test]
fn test_no_space_option() {
    let opts = ConvertOptions {
        space_after_terminal_punct: false,
        ..Default::default()
    };
    let result = convert_text_with_options("你好，世界", &opts);
    assert_eq!(result.text, "你好,世界");
}

#[test]
fn test_quotes_disabled_option() {
    let opts = ConvertOptions {
        convert_quotes: false,
        ..Default::default()
    };
    let result = convert_text_with_options("我说：“你好”，他说：‘世界’", &opts);
    assert_eq!(result.text, "我说:“你好”, 他说:‘世界’");
}

#[test]
fn test_leaves_ascii_untouched() {
    let input = "hello, world! 123 { test: [1, 2] }";
    let result = convert_text(input);
    assert_eq!(result.text, input);
    assert_eq!(result.replacements, 0);
}

#[test]
fn test_utf8_roundtrip() {
    let bytes = "你好，世界".as_bytes();
    let (text, enc) = decode(bytes).unwrap();
    assert_eq!(enc, DetectedEncoding::Utf8 { bom: false });
    let back = encode(&text, enc, false);
    assert_eq!(back, bytes);
}

#[test]
fn test_utf8_bom_roundtrip() {
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice("你好".as_bytes());
    let (text, enc) = decode(&bytes).unwrap();
    assert_eq!(text, "你好");
    assert_eq!(enc, DetectedEncoding::Utf8 { bom: true });
    let back = encode(&text, enc, false);
    assert_eq!(back, bytes);
}

#[test]
fn test_gb18030_roundtrip() {
    let (gb_bytes, _, had_errors) = GB18030.encode("你好，世界");
    assert!(!had_errors);
    let (text, enc) = decode(&gb_bytes).unwrap();
    assert_eq!(text, "你好，世界");
    assert_eq!(enc, DetectedEncoding::Gb18030);
    let back = encode(&text, enc, false);
    assert_eq!(back, gb_bytes.into_owned());
}

#[test]
fn test_binary_is_rejected() {
    let invalid_encoding: [u8; 4] = [0xFF, 0xFE, 0x00, 0x01];
    assert_eq!(decode(&invalid_encoding), Err(DecodeError::LikelyBinary));

    // Valid ASCII/UTF-8 bytes containing a NUL byte must also be rejected as binary.
    let nul_containing = b"hello\0\xEF\xBC\x8Cworld";
    assert_eq!(decode(nul_containing), Err(DecodeError::LikelyBinary));
}
