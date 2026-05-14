//! Parse tools - B-W12/02: Stream parsing for JSON, XML, Markdown

use super::ToolError;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use quick_xml::events::Event as XmlEvent;
use quick_xml::reader::Reader as XmlReader;
use serde_json::Value;
use std::io::Read;
use std::path::Path;

pub fn parse_json_stream<R: Read>(reader: R) -> Result<Value, ToolError> {
    serde_json::from_reader(reader).map_err(|e| ToolError::parse_error(format!("JSON: {}", e)))
}

pub fn parse_json_file<P: AsRef<Path>>(path: P) -> Result<Value, ToolError> {
    let file = std::fs::File::open(path).map_err(|e| ToolError::new(format!("Open: {}", e)))?;
    parse_json_stream(file)
}

pub struct XmlNode {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub text: String,
}

pub struct XmlParser<R: Read> {
    reader: XmlReader<R>,
    buf: Vec<u8>,
}

impl<R: std::io::BufRead> XmlParser<R> {
    pub fn new(reader: R) -> Self {
        let mut r = XmlReader::from_reader(reader);
        r.config_mut().trim_text(true);
        Self {
            reader: r,
            buf: Vec::with_capacity(1024),
        }
    }
}

impl<R: std::io::BufRead> Iterator for XmlParser<R> {
    type Item = Result<XmlNode, ToolError>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(XmlEvent::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let attrs = e
                        .attributes()
                        .filter_map(|a| {
                            let a = a.ok()?;
                            Some((
                                String::from_utf8_lossy(a.key.as_ref()).to_string(),
                                String::from_utf8_lossy(&a.value).to_string(),
                            ))
                        })
                        .collect();
                    return Some(Ok(XmlNode {
                        name,
                        attrs,
                        text: String::new(),
                    }));
                }
                Ok(XmlEvent::Text(e)) => {
                    return Some(Ok(XmlNode {
                        name: "#text".to_string(),
                        attrs: vec![],
                        text: e.unescape().ok()?.to_string(),
                    }));
                }
                Ok(XmlEvent::Eof) => return None,
                Err(e) => return Some(Err(ToolError::parse_error(format!("XML: {}", e)))),
                _ => {}
            }
        }
    }
}

pub fn parse_xml_file<P: AsRef<Path>>(path: P) -> Result<Vec<XmlNode>, ToolError> {
    use std::io::BufReader;
    let file = std::fs::File::open(path).map_err(|e| ToolError::new(format!("Open: {}", e)))?;
    XmlParser::new(BufReader::new(file)).collect()
}

#[derive(Debug)]
pub struct MarkdownItem {
    pub kind: ItemKind,
    pub content: String,
    pub url: Option<String>,
}
#[derive(Debug)]
pub enum ItemKind {
    Heading(u8),
    Link,
    Code,
    Text,
}

pub struct MarkdownParser<'a> {
    parser: Parser<'a>,
}

impl<'a> MarkdownParser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            parser: Parser::new(text),
        }
    }
}

impl<'a> Iterator for MarkdownParser<'a> {
    type Item = MarkdownItem;
    fn next(&mut self) -> Option<Self::Item> {
        let mut content = String::new();
        let mut url = None;
        let mut kind = None;

        for event in self.parser.by_ref() {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    kind = Some(ItemKind::Heading(level as u8));
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(k) = kind.take() {
                        return Some(MarkdownItem {
                            kind: k,
                            content: content.trim().to_string(),
                            url,
                        });
                    }
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    kind = Some(ItemKind::Link);
                    url = Some(dest_url.to_string());
                }
                Event::End(TagEnd::Link) if matches!(kind, Some(ItemKind::Link)) => {
                    return Some(MarkdownItem {
                        kind: ItemKind::Link,
                        content: content.trim().to_string(),
                        url,
                    });
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    kind = Some(ItemKind::Code);
                }
                Event::End(TagEnd::CodeBlock) if matches!(kind, Some(ItemKind::Code)) => {
                    return Some(MarkdownItem {
                        kind: ItemKind::Code,
                        content: content.trim().to_string(),
                        url: None,
                    });
                }
                Event::Text(text) => {
                    content.push_str(&text);
                }
                Event::Code(code) if matches!(kind, Some(ItemKind::Code)) => {
                    content.push_str(&code);
                }
                _ => {}
            }
        }
        None
    }
}

pub fn parse_markdown(text: &str) -> Vec<MarkdownItem> {
    MarkdownParser::new(text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_json_valid() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"key": "value"}"#;
        let v: Value = parse_json_stream(json.as_bytes())?;
        assert_eq!(v["key"], "value");
        Ok(())
    }
    #[test]
    fn test_parse_markdown_heading() {
        let md = "# Title\n## Subtitle";
        let items = parse_markdown(md);
        assert!(items
            .iter()
            .any(|i| matches!(i.kind, ItemKind::Heading(1)) && i.content == "Title"));
    }
}
