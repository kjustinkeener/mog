//! XML *reading* into JSON, via the lightweight read-only `roxmltree` parser. Like
//! the other json_* readers this is reading only (XML in, JSON out), not a general
//! XML transformer. The mapping is the common, opinionated one:
//!   - attributes  -> `@name` keys
//!   - child elements -> keys by tag name; repeated tags become an array
//!   - text content -> `#text` (or, for an element with no attributes or child
//!     elements, the element's value is just its text string)

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::model::Step;

fn element_to_value(node: roxmltree::Node) -> Value {
    let has_attrs = node.attributes().next().is_some();
    let child_elems: Vec<roxmltree::Node> = node.children().filter(|n| n.is_element()).collect();
    let text: String = node
        .children()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect();
    let text = text.trim();

    // A leaf element (no attributes, no child elements) is just its text.
    if !has_attrs && child_elems.is_empty() {
        return Value::String(text.to_string());
    }

    let mut map = serde_json::Map::new();
    for a in node.attributes() {
        map.insert(
            format!("@{}", a.name()),
            Value::String(a.value().to_string()),
        );
    }
    for child in child_elems {
        let key = child.tag_name().name().to_string();
        let val = element_to_value(child);
        if let Some(existing) = map.get_mut(&key) {
            if let Value::Array(arr) = existing {
                arr.push(val);
            } else {
                let prev = existing.take();
                *existing = Value::Array(vec![prev, val]);
            }
        } else {
            map.insert(key, val);
        }
    }
    if !text.is_empty() {
        map.insert("#text".to_string(), Value::String(text.to_string()));
    }
    Value::Object(map)
}

/// `xml_to_json`: parse XML and emit it as JSON, wrapped in an object keyed by the
/// root element's tag. Pretty by default; `compact` for one line.
pub fn xml_to_json(input: &str, step: &Step) -> Result<String> {
    let doc =
        roxmltree::Document::parse(input).map_err(|e| anyhow!("xml_to_json: invalid XML: {e}"))?;
    let root = doc.root_element();
    let mut map = serde_json::Map::new();
    map.insert(root.tag_name().name().to_string(), element_to_value(root));
    let v = Value::Object(map);
    if step.get_bool("compact", false)? {
        serde_json::to_string(&v).map_err(|e| anyhow!("{e}"))
    } else {
        serde_json::to_string_pretty(&v).map_err(|e| anyhow!("{e}"))
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn xml_attr_escape(s: &str) -> String {
    xml_escape(s).replace('"', "&quot;")
}

fn scalar_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => String::new(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// Emit `<name>` for `v`, inverting the xml_to_json mapping: `@k` -> attribute,
/// `#text` -> text, other keys -> child elements, arrays repeat the element.
fn value_to_xml(name: &str, v: &Value, out: &mut String, indent: usize) {
    match v {
        Value::Array(a) => {
            for item in a {
                value_to_xml(name, item, out, indent);
            }
        }
        Value::Object(map) => {
            let mut attrs = String::new();
            let mut children: Vec<(&String, &Value)> = Vec::new();
            let mut text: Option<String> = None;
            for (k, val) in map {
                if let Some(a) = k.strip_prefix('@') {
                    attrs.push_str(&format!(
                        " {}=\"{}\"",
                        a,
                        xml_attr_escape(&scalar_text(val))
                    ));
                } else if k == "#text" {
                    text = Some(scalar_text(val));
                } else {
                    children.push((k, val));
                }
            }
            let pad = " ".repeat(indent);
            if children.is_empty() {
                match text {
                    Some(t) if !t.is_empty() => out.push_str(&format!(
                        "{pad}<{name}{attrs}>{}</{name}>\n",
                        xml_escape(&t)
                    )),
                    _ => out.push_str(&format!("{pad}<{name}{attrs}/>\n")),
                }
            } else {
                out.push_str(&format!("{pad}<{name}{attrs}>\n"));
                for (k, val) in children {
                    value_to_xml(k, val, out, indent + 2);
                }
                out.push_str(&format!("{pad}</{name}>\n"));
            }
        }
        other => {
            let pad = " ".repeat(indent);
            out.push_str(&format!(
                "{pad}<{name}>{}</{name}>\n",
                xml_escape(&scalar_text(other))
            ));
        }
    }
}

/// `json_to_xml`: emit JSON as XML, inverting `xml_to_json`. The top level must be
/// an object; a single key is used as the root element (else the object is wrapped
/// in `<root>`).
pub fn json_to_xml(input: &str, _step: &Step) -> Result<String> {
    let v: Value =
        serde_json::from_str(input).map_err(|e| anyhow!("json_to_xml: invalid JSON: {e}"))?;
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow!("json_to_xml: top level must be a JSON object"))?;
    let mut out = String::new();
    if obj.len() == 1 {
        let (k, val) = obj.iter().next().unwrap();
        value_to_xml(k, val, &mut out, 0);
    } else {
        value_to_xml("root", &v, &mut out, 0);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn step(opts: Value) -> Step {
        Step {
            description: None,
            section: None,
            action: None,
            disabled: false,
            only_lines_matching: None,
            except_lines_matching: None,
            match_ignore_case: false,
            scope: None,
            options: opts.as_object().cloned().unwrap_or_default(),
        }
    }

    #[test]
    fn xml_attrs_children_and_repeats() {
        let xml = r#"<order id="7"><item>Pen</item><item>Ink</item><note>urgent</note></order>"#;
        let out = xml_to_json(xml, &step(json!({"compact":true}))).unwrap();
        assert_eq!(
            out,
            r#"{"order":{"@id":"7","item":["Pen","Ink"],"note":"urgent"}}"#
        );
    }

    #[test]
    fn json_to_xml_roundtrips() {
        let j = r#"{"order":{"@id":"7","item":["Pen","Ink"],"note":"urgent"}}"#;
        let xml = json_to_xml(j, &step(json!({}))).unwrap();
        // Re-reading the emitted XML yields the original JSON.
        assert_eq!(
            xml_to_json(&xml, &step(json!({"compact":true}))).unwrap(),
            j
        );
    }
}
