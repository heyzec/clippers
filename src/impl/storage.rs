use crate::r#impl::path;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::io::{BufReader, Write};

const EXTERNALIZE_THRESHOLD: usize = 1024; // 1KB

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum ContentRef {
    Inline(String),       // plain text (valid UTF-8)
    InlineBase64(String), // base64-encoded content (binary or non-UTF-8)
    External(String),     // content hash for lookup
}

pub struct ClipboardEntry {
    pub id: u64,
    pub timestamp: std::time::SystemTime,
    pub types: HashMap<String, Vec<u8>>, // mime_type -> content (bytes)
}

#[derive(Serialize, Deserialize)]
pub struct Storage {
    entries: VecDeque<ClipboardEntry>,
    highest_id: u64,
    #[serde(skip)]
    max_entries: usize,
}

impl Storage {
    pub fn compute_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }

    pub fn write_blob(hash: &str, content: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let blobs_dir = path::get_blobs_dir()?;
        let blob_path = blobs_dir.join(hash);

        // Only write if it doesn't exist (content-addressable)
        if !blob_path.exists() {
            let mut file = File::create(blob_path)?;
            file.write_all(content)?;
        }
        Ok(())
    }

    pub fn read_blob(hash: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let blobs_dir = path::get_blobs_dir()?;
        let blob_path = blobs_dir.join(hash);
        Ok(fs::read(blob_path)?)
    }

    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
            highest_id: 0,
        }
    }

    pub fn from_file(max_entries: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = path::get_history_file_path()?;

        let storage = if file_path.exists() {
            let file = File::open(&file_path)?;
            let reader = BufReader::new(file);
            let mut storage: Storage = serde_json::from_reader(reader)?;
            storage.max_entries = max_entries;
            storage
        } else {
            Storage::new(max_entries)
        };

        Ok(storage)
    }

    pub fn add_entry(&mut self, types: HashMap<String, Vec<u8>>) {
        self.highest_id += 1;
        let entry = ClipboardEntry::new(self.highest_id, types);

        self.entries.push_front(entry);

        // Trim to max_entries if needed
        if self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }

        let _ = self.to_file();
    }

    pub fn get_entry_by_id(&self, id: u64) -> Option<&ClipboardEntry> {
        // TODO: Consider using a hash map for faster lookup
        self.entries.iter().find(|entry| entry.id == id)
    }

    #[allow(dead_code)]
    pub fn get_entries(&self) -> &VecDeque<ClipboardEntry> {
        &self.entries
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = path::get_history_file_path()?;

        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = File::create(&file_path)?;
        serde_json::to_writer_pretty(file, &self)?;

        Ok(())
    }
}

impl Serialize for ClipboardEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ClipboardEntry", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("timestamp", &self.timestamp)?;

        // Build ContentRef map for types
        let mut types_refs = HashMap::new();
        for (k, v) in &self.types {
            let content_ref = if v.len() > EXTERNALIZE_THRESHOLD {
                // Externalize: compute hash and write blob
                let hash = Storage::compute_hash(v);
                if let Err(e) = Storage::write_blob(&hash, v) {
                    return Err(serde::ser::Error::custom(format!(
                        "Failed to write blob: {}",
                        e
                    )));
                }
                ContentRef::External(hash)
            } else {
                // Check if content is valid UTF-8 text
                match std::str::from_utf8(v) {
                    Ok(text) => {
                        // Valid UTF-8: store as plain text
                        ContentRef::Inline(text.to_string())
                    }
                    Err(_) => {
                        // Invalid UTF-8 or binary: use base64
                        let encoded = general_purpose::STANDARD.encode(v);
                        ContentRef::InlineBase64(encoded)
                    }
                }
            };
            types_refs.insert(k.clone(), content_ref);
        }

        state.serialize_field("types", &types_refs)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ClipboardEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct ClipboardEntryVisitor;

        impl<'de> Visitor<'de> for ClipboardEntryVisitor {
            type Value = ClipboardEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ClipboardEntry")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ClipboardEntry, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut timestamp = None;
                let mut types: Option<HashMap<String, ContentRef>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "timestamp" => {
                            if timestamp.is_some() {
                                return Err(de::Error::duplicate_field("timestamp"));
                            }
                            timestamp = Some(map.next_value()?);
                        }
                        "types" => {
                            if types.is_some() {
                                return Err(de::Error::duplicate_field("types"));
                            }
                            types = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let timestamp = timestamp.ok_or_else(|| de::Error::missing_field("timestamp"))?;
                let types_refs = types.ok_or_else(|| de::Error::missing_field("types"))?;

                // Convert ContentRef map to Vec<u8> map
                let types: HashMap<String, Vec<u8>> = types_refs
                    .into_iter()
                    .map(|(k, content_ref)| match content_ref {
                        ContentRef::Inline(text) => Ok((k, text.into_bytes())),
                        ContentRef::InlineBase64(encoded) => general_purpose::STANDARD
                            .decode(&encoded)
                            .map(|bytes| (k, bytes))
                            .map_err(de::Error::custom),
                        ContentRef::External(hash) => Storage::read_blob(&hash)
                            .map(|bytes| (k, bytes))
                            .map_err(|e| {
                                de::Error::custom(format!("Failed to read blob {}: {}", hash, e))
                            }),
                    })
                    .collect::<Result<_, _>>()?;

                Ok(ClipboardEntry {
                    id,
                    timestamp,
                    types,
                })
            }
        }

        const FIELDS: &[&str] = &["id", "timestamp", "types"];
        deserializer.deserialize_struct("ClipboardEntry", FIELDS, ClipboardEntryVisitor)
    }
}

impl ClipboardEntry {
    pub fn new(id: u64, types: HashMap<String, Vec<u8>>) -> Self {
        Self {
            id,
            timestamp: std::time::SystemTime::now(),
            types,
        }
    }

    pub fn get_content_by_type(&self, mime_type: &str) -> Option<&Vec<u8>> {
        self.types.get(mime_type)
    }

    pub fn get_text_content(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            self.get_content_by_type("text/plain")
                .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
        }

        #[cfg(target_os = "macos")]
        {
            self.get_content_by_type("public.utf8-plain-text")
                .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
        }
    }

    #[allow(dead_code)]
    pub fn get_available_types(&self) -> Vec<&String> {
        self.types.keys().collect()
    }

    pub fn get_binary_info(&self) -> String {
        for (mime_type, content) in &self.types {
            if mime_type == "public.utf8-plain-text" || content.len() == 0 {
                continue;
            }

            let size = content.len();
            let size_str = if size < 1024 {
                format!("{} B", size)
            } else if size < 1024 * 1024 {
                format!("{} KiB", size / 1024)
            } else {
                format!("{:.1} MiB", size as f64 / (1024.0 * 1024.0))
            };

            // Try to detect image format and dimensions using imagesize crate
            let (format, dims) = match imagesize::blob_size(content) {
                Ok(size) => {
                    // Get format from image type
                    let format_str = match imagesize::image_type(content) {
                        Ok(img_type) => {
                            use imagesize::ImageType;
                            match img_type {
                                ImageType::Png => "png",
                                ImageType::Jpeg => "jpg",
                                ImageType::Gif => "gif",
                                ImageType::Webp => "webp",
                                ImageType::Bmp => "bmp",
                                ImageType::Ico => "ico",
                                ImageType::Tiff => "tiff",
                                ImageType::Heif(_) => "heic",
                                ImageType::Qoi => "qoi",
                                ImageType::Tga => "tga",
                                ImageType::Pnm => "pnm",
                                ImageType::Hdr => "hdr",
                                ImageType::Exr => "exr",
                                ImageType::Farbfeld => "farbfeld",
                                ImageType::Psd => "psd",
                                ImageType::Aseprite => "ase",
                                ImageType::Ilbm => "ilbm",
                                ImageType::Vtf => "vtf",
                                _ => "image",
                            }
                        }
                        Err(_) => "image",
                    };
                    (format_str, Some((size.width, size.height)))
                }
                Err(_) => ("binary", None),
            };

            let dims_str = dims.map(|(w, h)| format!("{}x{}", w, h));

            let info = if let Some(dims) = dims_str {
                format!("[[ binary data {} {} {} ]]", size_str, format, dims)
            } else {
                format!("[[ binary data {} {} ]]", size_str, format)
            };
            return info;
        }
        "[no content available]".to_string()
    }
}
