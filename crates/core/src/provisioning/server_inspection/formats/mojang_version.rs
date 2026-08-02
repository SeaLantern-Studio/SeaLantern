use std::collections::BTreeMap;

use serde_json::{Map, Value};

use super::super::model::{MinecraftPackVersion, PackFormatVersion};

pub(crate) struct MojangVersionDocument {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) world_version: Option<i64>,
    pub(crate) series_id: Option<String>,
    pub(crate) protocol_version: Option<i64>,
    pub(crate) pack_version: Option<MinecraftPackVersion>,
    pub(crate) build_time: Option<String>,
    pub(crate) java_component: Option<String>,
    pub(crate) java_version: Option<u16>,
    pub(crate) stable: Option<bool>,
    pub(crate) use_editor: Option<bool>,
    pub(crate) extra: BTreeMap<String, Value>,
}

pub(crate) fn parse(content: &[u8]) -> Result<Option<MojangVersionDocument>, serde_json::Error> {
    let value: Value = serde_json::from_slice(content)?;
    let Value::Object(mut object) = value else {
        return Ok(None);
    };
    if !looks_like_mojang_version(&object) {
        return Ok(None);
    }

    let id = take_string(&mut object, "id");
    let name = take_string(&mut object, "name");
    let world_version = take_i64(&mut object, "world_version");
    let series_id = take_string(&mut object, "series_id");
    let protocol_version = take_i64(&mut object, "protocol_version");
    let pack_version = object.get("pack_version").and_then(parse_pack_version);
    if pack_version.is_some() {
        object.remove("pack_version");
    }
    let build_time = take_string(&mut object, "build_time");
    let java_component = take_string(&mut object, "java_component");
    let java_version = take_u16(&mut object, "java_version");
    let stable = take_bool(&mut object, "stable");
    let use_editor = take_bool(&mut object, "use_editor");

    Ok(Some(MojangVersionDocument {
        id,
        name,
        world_version,
        series_id,
        protocol_version,
        pack_version,
        build_time,
        java_component,
        java_version,
        stable,
        use_editor,
        extra: object.into_iter().collect(),
    }))
}

fn looks_like_mojang_version(object: &Map<String, Value>) -> bool {
    object.get("id").is_some_and(Value::is_string)
        && ["world_version", "protocol_version", "pack_version", "java_version"]
            .iter()
            .any(|key| object.contains_key(*key))
}

fn parse_pack_version(value: &Value) -> Option<MinecraftPackVersion> {
    let object = value.as_object()?;
    let resource = parse_named_pack_version(object, "resource", "resource_major", "resource_minor");
    let data = parse_named_pack_version(object, "data", "data_major", "data_minor");
    (resource.is_some() || data.is_some()).then_some(MinecraftPackVersion { resource, data })
}

fn parse_named_pack_version(
    object: &Map<String, Value>,
    short_name: &str,
    major_name: &str,
    minor_name: &str,
) -> Option<PackFormatVersion> {
    if let Some(major) = object.get(major_name).and_then(value_as_u32) {
        return Some(PackFormatVersion {
            major,
            minor: object.get(minor_name).and_then(value_as_u32).unwrap_or(0),
        });
    }

    match object.get(short_name)? {
        Value::Number(number) => u32::try_from(number.as_u64()?)
            .ok()
            .map(|major| PackFormatVersion { major, minor: 0 }),
        Value::Object(nested) => Some(PackFormatVersion {
            major: nested.get("major").and_then(value_as_u32)?,
            minor: nested.get("minor").and_then(value_as_u32).unwrap_or(0),
        }),
        _ => None,
    }
}

fn take_string(object: &mut Map<String, Value>, key: &str) -> Option<String> {
    let value = object.get(key)?.as_str()?.to_string();
    object.remove(key);
    Some(value)
}

fn take_i64(object: &mut Map<String, Value>, key: &str) -> Option<i64> {
    let value = object.get(key)?.as_i64()?;
    object.remove(key);
    Some(value)
}

fn take_u16(object: &mut Map<String, Value>, key: &str) -> Option<u16> {
    let value = u16::try_from(object.get(key)?.as_u64()?).ok()?;
    object.remove(key);
    Some(value)
}

fn take_bool(object: &mut Map<String, Value>, key: &str) -> Option<bool> {
    let value = object.get(key)?.as_bool()?;
    object.remove(key);
    Some(value)
}

fn value_as_u32(value: &Value) -> Option<u32> {
    u32::try_from(value.as_u64()?).ok()
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parses_current_mojang_version_shape_and_keeps_unknown_fields() {
        let parsed = parse(
            br#"{
                "id":"26.2",
                "name":"26.2",
                "world_version":4903,
                "protocol_version":776,
                "pack_version":{"resource_major":88,"resource_minor":0,"data_major":107,"data_minor":1},
                "java_version":25,
                "custom":"kept"
            }"#,
        )
        .expect("valid JSON")
        .expect("Mojang version document");

        assert_eq!(parsed.id.as_deref(), Some("26.2"));
        assert_eq!(parsed.java_version, Some(25));
        let pack = parsed.pack_version.expect("pack version");
        assert_eq!(pack.resource.expect("resource format").major, 88);
        assert_eq!(pack.data.expect("data format").minor, 1);
        assert_eq!(parsed.extra.get("custom").and_then(|value| value.as_str()), Some("kept"));
    }
}
