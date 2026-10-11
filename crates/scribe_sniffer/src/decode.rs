use base64::{engine::general_purpose::STANDARD, Engine};
use flate2::read::ZlibDecoder;
use std::io::Read;

const MAX_DECOMPRESSED: u64 = 16 * 1024 * 1024;

fn looks_like_json_object(s: &str) -> bool {
    s.starts_with('{') && s.ends_with('}')
}


pub fn normalize_frame(frame: &str) -> Option<String> {
    let trimmed = frame.trim();

    if looks_like_json_object(trimmed) {
        return Some(trimmed.to_string());
    }

    let encoded = trimmed.strip_prefix('Z')?;
    let raw = STANDARD.decode(encoded).ok()?;

    let mut text = String::new();
    ZlibDecoder::new(&raw[..])
        .take(MAX_DECOMPRESSED)
        .read_to_string(&mut text)
        .ok()?;

    let text = text.trim();
    looks_like_json_object(text).then(|| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_json_passes_through() {
        assert_eq!(normalize_frame(" {\"a\":1} ").as_deref(), Some("{\"a\":1}"));
    }

    #[test]
    fn compressed_json_is_decoded() {
        let frame = "ZeJyrVipRslKqKFHSUUpSsqpWygcRybkpQMFix+QSpdraWgCxcgqF";
        assert_eq!(
            normalize_frame(frame).as_deref(),
            Some("{\"t\":\"xt\",\"b\":{\"o\":{\"cmd\":\"sAct\"}}}")
        );
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(normalize_frame("hello").is_none());
        assert!(normalize_frame("Znot base64!!").is_none());
        assert!(normalize_frame("ZaGVsbG8=").is_none()); // valid base64, not zlib
    }
}