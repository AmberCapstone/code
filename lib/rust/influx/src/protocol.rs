use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn line_protocol(
    measurement: &str,
    tags: &[String],
    fields: &[(String, serde_json::Value)],
    time: SystemTime,
) -> String {
    let tag_set = if tags.is_empty() {
        String::new()
    } else {
        format!(",{}", tags.join(","))
    };

    let field_set = fields
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",");
    let timestamp = time.duration_since(UNIX_EPOCH).unwrap().as_nanos();

    format!("{measurement}{tag_set} {field_set} {timestamp}")
}
