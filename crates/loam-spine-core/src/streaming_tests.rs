// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn data_item_roundtrip() {
    let item = StreamItem::data(serde_json::json!({"spine_id": "abc123"}));
    let line = item.to_ndjson_line().unwrap();
    assert!(line.ends_with('\n'));
    let parsed = StreamItem::parse_ndjson_line(&line).unwrap();
    assert_eq!(item, parsed);
}

#[test]
fn end_marker() {
    let item = StreamItem::end();
    assert!(item.is_end());
    assert!(!item.is_fatal());
    let line = item.to_ndjson_line().unwrap();
    let parsed = StreamItem::parse_ndjson_line(&line).unwrap();
    assert!(parsed.is_end());
}

#[test]
fn progress_with_total() {
    let item = StreamItem::progress(42, Some(100));
    let json = serde_json::to_value(&item).unwrap();
    assert_eq!(json["type"], "Progress");
    assert_eq!(json["processed"], 42);
    assert_eq!(json["total"], 100);
}

#[test]
fn progress_without_total() {
    let item = StreamItem::progress(7, None);
    let json = serde_json::to_value(&item).unwrap();
    assert!(json["total"].is_null());
}

#[test]
fn recoverable_error() {
    let item = StreamItem::error("transient failure");
    assert!(!item.is_fatal());
    assert!(!item.is_end());
    let json = serde_json::to_value(&item).unwrap();
    assert_eq!(json["recoverable"], true);
}

#[test]
fn fatal_error() {
    let item = StreamItem::fatal("corruption detected");
    assert!(item.is_fatal());
    let json = serde_json::to_value(&item).unwrap();
    assert_eq!(json["recoverable"], false);
}

#[test]
fn parse_invalid_ndjson() {
    assert!(StreamItem::parse_ndjson_line("not json").is_err());
}

#[test]
fn protocol_version_is_semver() {
    assert!(NDJSON_PROTOCOL_VERSION.contains('.'));
}

#[tokio::test]
async fn read_ndjson_stream_parses_items() {
    let input = format!(
        "{}{}{}\n",
        StreamItem::data(serde_json::json!({"id": 1}))
            .to_ndjson_line()
            .unwrap(),
        StreamItem::progress(1, Some(2)).to_ndjson_line().unwrap(),
        StreamItem::end().to_ndjson_line().unwrap().trim_end(),
    );
    let mut cursor = std::io::Cursor::new(input.as_bytes().to_vec());
    let mut reader = tokio::io::BufReader::new(&mut cursor);
    let items = super::read_ndjson_stream(&mut reader).await.unwrap();
    assert_eq!(items.len(), 3);
    assert!(items[2].is_end());
}

#[tokio::test]
async fn read_ndjson_stream_stops_on_fatal() {
    let input = format!(
        "{}{}",
        StreamItem::fatal("boom").to_ndjson_line().unwrap(),
        StreamItem::data(serde_json::json!({"after_fatal": true}))
            .to_ndjson_line()
            .unwrap(),
    );
    let mut cursor = std::io::Cursor::new(input.as_bytes().to_vec());
    let mut reader = tokio::io::BufReader::new(&mut cursor);
    let items = super::read_ndjson_stream(&mut reader).await.unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].is_fatal());
}

#[tokio::test]
async fn read_ndjson_stream_handles_parse_errors() {
    let input = "not valid json\n";
    let mut cursor = std::io::Cursor::new(input.as_bytes().to_vec());
    let mut reader = tokio::io::BufReader::new(&mut cursor);
    let items = super::read_ndjson_stream(&mut reader).await.unwrap();
    assert_eq!(items.len(), 1);
    assert!(matches!(
        items[0],
        StreamItem::Error {
            recoverable: true,
            ..
        }
    ));
}

#[test]
fn ndjson_multi_line_stream() {
    let items = vec![
        StreamItem::data(serde_json::json!({"id": 1})),
        StreamItem::data(serde_json::json!({"id": 2})),
        StreamItem::progress(2, Some(3)),
        StreamItem::data(serde_json::json!({"id": 3})),
        StreamItem::end(),
    ];

    let stream: String = items.iter().map(|i| i.to_ndjson_line().unwrap()).collect();

    let parsed: Vec<StreamItem> = stream
        .lines()
        .map(|line| StreamItem::parse_ndjson_line(line).unwrap())
        .collect();

    assert_eq!(items, parsed);
}

#[tokio::test]
async fn read_ndjson_stream_skips_empty_lines() {
    let input = format!(
        "\n\n{}\n\n{}\n",
        StreamItem::data(serde_json::json!({"ok": true}))
            .to_ndjson_line()
            .unwrap()
            .trim_end(),
        StreamItem::end().to_ndjson_line().unwrap().trim_end(),
    );
    let mut cursor = std::io::Cursor::new(input.as_bytes().to_vec());
    let mut reader = tokio::io::BufReader::new(&mut cursor);
    let items = super::read_ndjson_stream(&mut reader).await.unwrap();
    assert_eq!(items.len(), 2);
    assert!(matches!(items[0], StreamItem::Data { .. }));
    assert!(items[1].is_end());
}
