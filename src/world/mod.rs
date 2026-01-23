//! VirtualWorld - Typst World implementation for WASM
//!
//! This module implements the `typst::World` trait to enable Typst compilation
//! in the browser environment.

mod fonts;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::OnceLock;

use typst::diag::{FileError, FileResult, SourceDiagnostic};
use typst::foundations::{Bytes, Datetime};
use typst::layout::{Abs, Frame, FrameItem, Point, Size, Transform};
use typst::model::Destination;
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

use fonts::FontLoader;

/// Static library instance
static LIBRARY: OnceLock<LazyHash<Library>> = OnceLock::new();

/// Get or initialize the Typst standard library
fn library() -> &'static LazyHash<Library> {
    LIBRARY.get_or_init(|| LazyHash::new(Library::default()))
}

/// VirtualWorld implements typst::World for in-browser Typst compilation
pub struct VirtualWorld {
    /// The main source file
    main: Source,
    /// Virtual file system
    files: HashMap<FileId, Bytes>,
    /// Font loader
    font_loader: FontLoader,
    /// Font book
    font_book: LazyHash<FontBook>,
}

impl VirtualWorld {
    /// Create a new VirtualWorld with the given main source content
    pub fn new(source: &str) -> Self {
        let main_id = FileId::new(None, VirtualPath::new("main.typ"));
        let main = Source::new(main_id, source.to_string());

        let font_loader = FontLoader::new();
        let font_book = LazyHash::new(font_loader.font_book());

        Self {
            main,
            files: HashMap::new(),
            font_loader,
            font_book,
        }
    }

    /// Set the main source content
    #[allow(dead_code)]
    pub fn set_source(&mut self, source: &str) {
        let main_id = FileId::new(None, VirtualPath::new("main.typ"));
        self.main = Source::new(main_id, source.to_string());
    }

    /// Add a file to the virtual file system
    #[allow(dead_code)]
    pub fn add_file(&mut self, path: &str, content: impl Into<Bytes>) {
        let id = FileId::new(None, VirtualPath::new(path));
        self.files.insert(id, content.into());
    }

    /// Compile the current source to a Document
    pub fn compile(&self) -> Result<typst::model::Document, Vec<SourceDiagnostic>> {
        let result = typst::compile(self);
        result.output.map_err(|errs| errs.into_iter().collect())
    }

    /// Compile source text to SVG string (convenience method)
    pub fn compile_to_svg(source: &str) -> Result<String, Vec<String>> {
        let world = Self::new(source);

        match world.compile() {
            Ok(doc) => {
                if let Some(page) = doc.pages.first() {
                    let svg = typst_svg::svg(page);

                    // Extract links from the frame and add them to SVG
                    let links = extract_links_from_frame(&page.frame, Point::zero());

                    // Post-process SVG to add link overlays
                    let svg_with_links = add_links_to_svg(&svg, &links, page.frame.size());

                    Ok(svg_with_links)
                } else {
                    Err(vec!["Document has no pages".to_string()])
                }
            }
            Err(diagnostics) => {
                let errors: Vec<String> = diagnostics
                    .iter()
                    .map(|d| {
                        let severity = match d.severity {
                            typst::diag::Severity::Error => "Error",
                            typst::diag::Severity::Warning => "Warning",
                        };
                        format!("{}: {}", severity, d.message)
                    })
                    .collect();
                Err(errors)
            }
        }
    }

    /// Compile source text to SVG string with images from cache
    ///
    /// This method pre-populates the virtual file system with images
    /// from the ImageCache before compiling.
    #[cfg(any(target_arch = "wasm32", test))]
    pub fn compile_to_svg_with_images(
        source: &str,
        cache: &crate::images::ImageCache,
    ) -> Result<String, Vec<String>> {
        let mut world = Self::new(source);

        // Add images from cache to virtual file system
        cache.populate_world(&mut world);

        match world.compile() {
            Ok(doc) => {
                if let Some(page) = doc.pages.first() {
                    let svg = typst_svg::svg(page);

                    // Extract links from the frame and add them to SVG
                    let links = extract_links_from_frame(&page.frame, Point::zero());

                    // Post-process SVG to add link overlays
                    let svg_with_links = add_links_to_svg(&svg, &links, page.frame.size());

                    Ok(svg_with_links)
                } else {
                    Err(vec!["Document has no pages".to_string()])
                }
            }
            Err(diagnostics) => {
                let errors: Vec<String> = diagnostics
                    .iter()
                    .map(|d| {
                        let severity = match d.severity {
                            typst::diag::Severity::Error => "Error",
                            typst::diag::Severity::Warning => "Warning",
                        };
                        format!("{}: {}", severity, d.message)
                    })
                    .collect();
                Err(errors)
            }
        }
    }

    /// Get all available fonts
    #[allow(dead_code)]
    pub fn fonts(&self) -> &[Font] {
        self.font_loader.fonts()
    }
}

impl World for VirtualWorld {
    fn library(&self) -> &LazyHash<Library> {
        library()
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.font_book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            return Ok(self.main.clone());
        }
        // Check virtual files for .typ files
        let content = self
            .files
            .get(&id)
            .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))?;
        let text = std::str::from_utf8(content).map_err(|_| FileError::InvalidUtf8)?;
        Ok(Source::new(id, text.to_string()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.font_loader.font(index)
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        use chrono::{Datelike, Duration, Utc};
        let adjusted = Utc::now() + Duration::hours(offset.unwrap_or(0));
        Datetime::from_ymd(
            adjusted.year(),
            adjusted.month().try_into().ok()?,
            adjusted.day().try_into().ok()?,
        )
    }
}

/// A link extracted from the document with position and URL
#[derive(Debug)]
struct LinkInfo {
    /// Position relative to page origin
    x: f64,
    y: f64,
    /// Size of the link area
    width: f64,
    height: f64,
    /// Destination URL
    url: String,
}

/// Extract all links from a frame recursively
fn extract_links_from_frame(frame: &Frame, offset: Point) -> Vec<LinkInfo> {
    let mut links = Vec::new();

    for (pos, item) in frame.items() {
        let abs_pos = Point::new(offset.x + pos.x, offset.y + pos.y);

        match item {
            FrameItem::Link(Destination::Url(url), size) => {
                links.push(LinkInfo {
                    x: abs_pos.x.to_pt(),
                    y: abs_pos.y.to_pt(),
                    width: size.x.to_pt(),
                    height: size.y.to_pt(),
                    url: url.as_str().to_string(),
                });
            }
            FrameItem::Group(group) => {
                // Recursively extract links from nested groups
                let group_offset = apply_transform(abs_pos, group.transform);
                let nested_links = extract_links_from_frame(&group.frame, group_offset);
                links.extend(nested_links);
            }
            _ => {}
        }
    }

    links
}

/// Apply a transform to a point (simplified - only handles translation)
fn apply_transform(pos: Point, transform: Transform) -> Point {
    // Transform matrix is [[a, b, c], [d, e, f]] where c and f are translations
    Point::new(
        Abs::pt(pos.x.to_pt() + transform.tx.to_pt()),
        Abs::pt(pos.y.to_pt() + transform.ty.to_pt()),
    )
}

/// Add link overlay elements to the SVG
fn add_links_to_svg(svg: &str, links: &[LinkInfo], _page_size: Size) -> String {
    if links.is_empty() {
        return svg.to_string();
    }

    // Build link elements as SVG <a> tags with transparent rectangles
    let mut link_elements = String::new();
    for link in links {
        // Create a clickable rectangle for each link
        link_elements.push_str(&format!(
            r#"<a href="{}" target="_self"><rect x="{}" y="{}" width="{}" height="{}" fill="transparent" style="cursor: pointer;" /></a>"#,
            escape_xml(&link.url),
            link.x,
            link.y,
            link.width,
            link.height
        ));
    }

    // Insert link elements at the end of the SVG, just before the closing </svg> tag
    // This ensures they're on top and clickable
    if let Some(closing_idx) = svg.rfind("</svg>") {
        let mut result = svg[..closing_idx].to_string();
        result.push_str(&link_elements);
        result.push_str("</svg>");
        result
    } else {
        svg.to_string()
    }
}

/// Escape XML special characters in a string
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
