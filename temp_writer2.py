r'''#
#[derive(Debug, Clone, PartialEq)]
pub enum NerError { EmptyInput, InvalidUnicode(String) }

impl std::fmt::Display for NerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { 
            Self::EmptyInput => write!(f, "Empty input text"), 
            Self::InvalidUnicode(msg) => write!(f, "Invalid unicode: {}", msg) 
        }
    }
}

impl std::error::Error for NerError {}

impl Serialize for Entity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        use serde::ser::SerializeStruct; 
        let mut s = serializer.serialize_struct("Entity", 4)?;
        s.serialize_field("id", &self.id.to_string())?; 
        s.serialize_field("label", &self.label)?;
        s.serialize_field("span", &self.span)?; 
        s.serialize_field("confidence", &self.confidence)?; 
        s.end()
    }
}

impl<'de> Deserialize<'de> for Entity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        use serde::de::{self, MapAccess, Visitor}; 
        struct EntityVisitor;
        impl<'de> Visitor<'de> for EntityVisitor { 
            type Value = Entity;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result { 
                formatter.write_str("struct Entity") 
            }
            fn visit_map<V>(self, mut map: V) -> Result<Entity, V::Error> where V: MapAccess<'de> {
                let (mut id, mut label, mut span, mut conf) = (None, None, None, None);
                while let Some(k) = map.next_key::<String>()? {
                    match k.as_str() { 
                        "id" => { let s: String = map.next_value()?; id = Some(Uuid::parse_str(&s).map_err(de::Error::custom)?); }
                        "label" => label = Some(map.next_value()?), 
                        "span" => span = Some(map.next_value()?), 
                        "confidence" => conf = Some(map::next_value()?),
                        _ => { let _: serde_json::Value = map.next_value()?; } 
                    }
                }
                Ok(Entity { 
                    id: id.ok_or_else(|| de::Error::missing_field("id"))?,
                    label: label.ok_or_else(|| de::Error::missing_field("label"))?,
                    span: span.ok_or_else(|| de::Error::missing_field("span"))?,
                    confidence: conf.ok_or_else(|| de::Error::missing_field("confidence"))?, 
                })
            }
        }
        deserializer.deserialize_struct("Entity", &["id", "label", "span", "confidence"], EntityVisitor)
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}@({},{})", self.label, self.span.0, self.span.1)
    }
}

static AC_PATTERNS: &[( &str, &str, f32)] = &[
    ("Apple", "org", 0.95), ("Microsoft", "org", 0.95), ("Google", "org", 0.95),
    ("Amazon", "org", 0.95), ("Meta", "org", 0.95), ("Tesla", "org", 0.95),
    ("Steve Jobs", "person", 0.95), ("Bill Gates", "person", 0.95), ("Elon Musk", "person", 0.95),
    ("iPhone", "product", 0.95), ("iPad", "product", 0.95), ("MacBook", "product", 0.95),
    ("Windows", "product", 0.90), ("Android", "product", 0.90),
    ("Inc", "org_marker", 0.60), ("Corp", "org_marker", 0.60), ("Ltd", "org_marker", 0.60),
];

pub fn extract_entities(text: &str) -> Result<Vec<Entity>, NerError> {
    if text.is_empty() { return Err(NerError::EmptyInput); }
    if text.chars().any(|c| c == '\u{FFFD}') { return Err(NerError::InvalidUnicode("Replacement char".into())); }
    
    let patterns: Vec<&str> = AC_PATTERNS.iter().map(|(p, _, _)| *p).collect();
    let ac = AhoCorasick::new(&patterns).map_err(|_| NerError::InvalidUnicode("AC build failed".into()))?;
    
    let mut entities = Vec::new();
    for mat in ac.find_iter(text) {
        let pattern_idx = mat.pattern().as_usize();
        let (_, etype, base_conf) = AC_PATTERNS[pattern_idx];
        let len_bonus = (mat.len() as f32 / 20.0).min(0.05);
        let score = (base_conf + len_bonus).clamp(0.0, 1.0);
        if score > 0.3 && !etype.ends_with("_marker") {
            entities.push(Entity { 
                id: Uuid::new_v4(), 
                label: mat.as_str().to_string(), 
                span: (mat.start(), mat.end()), 
                confidence: score 
            });
        }
    }
    
    let regex_patterns: Vec<(&str, &str, f32)> = vec![
        (r"[A-Z][a-z]+(?:\\s+[A-Z][a-z]+)*", "person", 0.5),
        (r"(?:Company|Group|Tech|University)", "org", 0.6),
    ];
    for (pat, etype, base_conf) in regex_patterns {
        if let Ok(re) = Regex::new(pat) {
            for mat in re.find_iter(text) {
                let already_found = entities.iter().any(|e: &Entity| e.span.0 <= mat.start() && mat.end() <= e.span.1);
                if !already_found {
                    let len_bonus = (mat.as_str().len() as f32 / 20.0).min(0.1);
                    let score = (base_conf + len_bonus).clamp(0.0, 1.0);
                    if score > 0.3 { 
                        entities.push(Entity { id: Uuid::new_v4(), label: mat.as_str().to_string(), span: (mat.start(), mat.end()), confidence: score }); 
                    }
                }
            }
        }
    }
    
    entities.sort_by(|a, b| a.span.0.cmp(&b.span.0)); 
    entities.dedup_by(|a, b| a.span == b.span); 
    Ok(entities)
}

pub fn batch_extract(texts: &[&str]) -> Vec<Result<Vec<Entity>, NerError>> {
    texts.iter().map(|t| extract_entities(t)).collect()
}
'''

with open('F:/hajimi-code-cli/temp_writer2.py', 'w') as f:
    f.write(r'''
content = r"""
#[derive(Debug, Clone, PartialEq)]
pub enum NerError { EmptyInput, InvalidUnicode(String) }

impl std::fmt::Display for NerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { 
            Self::EmptyInput => write!(f, "Empty input text"), 
            Self::InvalidUnicode(msg) => write!(f, "Invalid unicode: {}", msg) 
        }
    }
}

impl std::error::Error for NerError {}

impl Serialize for Entity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        use serde::ser::SerializeStruct; 
        let mut s = serializer.serialize_struct("Entity", 4)?;
        s.serialize_field("id", &self.id.to_string())?; 
        s.serialize_field("label", &self.label)?;
        s.serialize_field("span", &self.span)?; 
        s.serialize_field("confidence", &self.confidence)?; 
        s.end()
    }
}

impl<'de> Deserialize<'de> for Entity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        use serde::de::{self, MapAccess, Visitor}; 
        struct EntityVisitor;
        impl<'de> Visitor<'de> for EntityVisitor { 
            type Value = Entity;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result { 
                formatter.write_str("struct Entity") 
            }
            fn visit_map<V>(self, mut map: V) -> Result<Entity, V::Error> where V: MapAccess<'de> {
                let (mut id, mut label, mut span, mut conf) = (None, None, None, None);
                while let Some(k) = map.next_key::<String>()? {
                    match k.as_str() { 
                        "id" => { let s: String = map.next_value()?; id = Some(Uuid::parse_str(&s).map_err(de::Error::custom)?); }
                        "label" => label = Some(map.next_value()?), 
                        "span" => span = Some(map.next_value()?), 
                        "confidence" => conf = Some(map.next_value()?),
                        _ => { let _: serde_json::Value = map.next_value()?; } 
                    }
                }
                Ok(Entity { 
                    id: id.ok_or_else(|| de::Error::missing_field("id"))?,
                    label: label.ok_or_else(|| de::Error::missing_field("label"))?,
                    span: span.ok_or_else(|| de::Error::missing_field("span"))?,
                    confidence: conf.ok_or_else(|| de::Error::missing_field("confidence"))?, 
                })
            }
        }
        deserializer.deserialize_struct("Entity", &["id", "label", "span", "confidence"], EntityVisitor)
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { 
        write!(f, "{}@({},{})", self.label, self.span.0, self.span.1)
    }
}

static AC_PATTERNS: &[(&str, &str, f32)] = &[
    ("Apple", "org", 0.95), ("Microsoft", "org", 0.95), ("Google", "org", 0.95),
    ("Amazon", "org", 0.95), ("Meta", "org", 0.95), ("Tesla", "org", 0.95),
    ("Steve Jobs", "person", 0.95), ("Bill Gates", "person", 0.95), ("Elon Musk", "person", 0.95),
    ("iPhone", "product", 0.95), ("iPad", "product", 0.95), ("MacBook", "product", 0.95),
    ("Windows", "product", 0.90), ("Android", "product", 0.90),
    ("Inc", "org_marker", 0.60), ("Corp", "org_marker", 0.60), ("Ltd", "org_marker", 0.60),
];

pub fn extract_entities(text: &str) -> Result<Vec<Entity>, NerError> {
    if text.is_empty() { return Err(NerError::EmptyInput); }
    if text.chars().any(|c| c == '\u{FFFD}') { return Err(NerError::InvalidUnicode("Replacement char".into())); }
    
    let patterns: Vec<&str> = AC_PATTERNS.iter().map(|(p, _, _)| *p).collect();
    let ac = AhoCorasick::new(&patterns).map_err(|_| NerError::InvalidUnicode("AC build failed".into()))?;
    
    let mut entities = Vec::new();
    for mat in ac.find_iter(text) {
        let pattern_idx = mat.pattern().as_usize();
        let (_, etype, base_conf) = AC_PATTERNS[pattern_idx];
        let len_bonus = (mat.len() as f32 / 20.0).min(0.05);
        let score = (base_conf + len_bonus).clamp(0.0, 1.0);
        if score > 0.3 && !etype.ends_with("_marker") {
            entities.push(Entity { 
                id: Uuid::new_v4(), 
                label: mat.as_str().to_string(), 
                span: (mat.start(), mat.end()), 
                confidence: score 
            });
        }
    }
    
    let regex_patterns: Vec<(&str, &str, f32)> = vec![
        (r"[A-Z][a-z]+(?:\\s+[A-Z][a-z]+)*", "person", 0.5),
        (r"(?:Company|Group|Tech|University)", "org", 0.6),
    ];
    for (pat, etype, base_conf) in regex_patterns {
        if let Ok(re) = Regex::new(pat) {
            for mat in re.find_iter(text) {
                let already_found = entities.iter().any(|e: &Entity| e.span.0 <= mat.start() && mat.end() <= e.span.1);
                if !already_found {
                    let len_bonus = (mat.as_str().len() as f32 / 20.0).min(0.1);
                    let score = (base_conf + len_bonus).clamp(0.0, 1.0);
                    if score > 0.3 { 
                        entities.push(Entity { id: Uuid::new_v4(), label: mat.as_str().to_string(), span: (mat.start(), mat.end()), confidence: score }); 
                    }
                }
            }
        }
    }
    
    entities.sort_by(|a, b| a.span.0.cmp(&b.span.0)); 
    entities.dedup_by(|a, b| a.span == b.span); 
    Ok(entities)
}

pub fn batch_extract(texts: &[&str]) -> Vec<Result<Vec<Entity>, NerError>> {
    texts.iter().map(|t| extract_entities(t)).collect()
}
"""
with open('F:/hajimi-code-cli/src/intelligence/memory/src/graph.rs', 'a') as f:
    f.write(content)
print('Part 2 written')
''')

print('Script created')
