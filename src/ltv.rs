use std::fs::OpenOptions;
use std::io::Write;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Mutex, MutexGuard};
use std::sync::Arc;

use cursive::{Printer, Vec2, With, XY};
use cursive::align::*;
use cursive::theme::*;
use cursive::utils::lines::spans::{LinesIterator, Row};
use cursive::utils::markup::StyledString;
use cursive::view::{SizeCache, View};
use lru::LruCache;
use owning_ref::{ArcRef, OwningHandle};
use unicode_width::UnicodeWidthStr;

// Content type used internally for caching and storage
type InnerContentType = Arc<StyledString>;

fn log<S>(line: S) where S: AsRef<str> {
    let mut f = OpenOptions::new()
        .append(true)
        .create(true)
        .open("/tmp/siv.log")
        .unwrap();
    f.write_all(line.as_ref().as_bytes()).unwrap();
    f.write_all(b"\n").unwrap();
}

/// Provides access to the content of a [`TextView`].
///
/// [`TextView`]: struct.TextView.html
///
/// Cloning this object will still point to the same content.
///
/// # Examples
///
/// ```rust
/// # use cursive_core::views::{TextView, TextContent};
/// let mut content = TextContent::new("content");
/// let view = TextView::new_with_content(content.clone());
///
/// // Later, possibly in a different thread
/// content.set_content("new content");
/// assert!(view.get_content().source().contains("new"));
/// ```
#[derive(Clone)]
pub struct TextContent {
    content: Arc<Mutex<TextContentInner>>,
}

impl TextContent {
    /// Creates a new text content around the given value.
    ///
    /// Parses the given value.
    pub fn new<S>(content: S) -> Self
        where
            S: Into<StyledString>,
    {
        let content = Arc::new(content.into());

        TextContent {
            content: Arc::new(Mutex::new(TextContentInner {
                content_value: content,
                content_cache: Arc::new(StyledString::default()),
                size_cache: None,
            })),
        }
    }
}

/// A reference to the text content.
///
/// This can be deref'ed into a [`StyledString`].
///
/// [`StyledString`]: ../utils/markup/type.StyledString.html
///
/// This keeps the content locked. Do not store this!
pub struct TextContentRef {
    _handle: OwningHandle<
        ArcRef<Mutex<TextContentInner>>,
        MutexGuard<'static, TextContentInner>,
    >,
    // We also need to keep a copy of Arc so `deref` can return
    // a reference to the `StyledString`
    data: Arc<StyledString>,
}

impl Deref for TextContentRef {
    type Target = StyledString;

    fn deref(&self) -> &StyledString {
        self.data.as_ref()
    }
}

impl TextContent {
    /// Replaces the content with the given value.
    pub fn set_content<S>(&self, content: S)
        where
            S: Into<StyledString>,
    {
        self.with_content(|c| {
            *c = content.into();
        });
    }

    /// Append `content` to the end of a `TextView`.
    pub fn append<S>(&self, content: S)
        where
            S: Into<StyledString>,
    {
        self.with_content(|c| {
            // This will only clone content if content_cached and content_value
            // are sharing the same underlying Rc.
            c.append(content);
        })
    }

    /// Returns a reference to the content.
    ///
    /// This locks the data while the returned value is alive,
    /// so don't keep it too long.
    pub fn get_content(&self) -> TextContentRef {
        TextContentInner::get_content(&self.content)
    }

    /// Apply the given closure to the inner content, and bust the cache afterward.
    pub fn with_content<F, O>(&self, f: F) -> O
        where
            F: FnOnce(&mut StyledString) -> O,
    {
        self.with_content_inner(|c| f(Arc::make_mut(&mut c.content_value)))
    }

    /// Apply the given closure to the inner content, and bust the cache afterward.
    fn with_content_inner<F, O>(&self, f: F) -> O
        where
            F: FnOnce(&mut TextContentInner) -> O,
    {
        let mut content = self.content.lock().unwrap();

        let out = f(&mut content);

        content.size_cache = None;

        out
    }
}


struct TextContentInner {
    content_value: InnerContentType,
    content_cache: InnerContentType,

    // We keep the cache here so it can be busted when we change the content.
    size_cache: Option<XY<SizeCache>>,
}

impl TextContentInner {
    /// From a shareable content (Arc + Mutex), return a
    fn get_content(content: &Arc<Mutex<TextContentInner>>) -> TextContentRef {
        let arc_ref: ArcRef<Mutex<TextContentInner>> =
            ArcRef::new(Arc::clone(content));

        let _handle = OwningHandle::new_with_fn(arc_ref, |mutex| unsafe {
            (*mutex).lock().unwrap()
        });

        let data = Arc::clone(&_handle.content_value);

        TextContentRef { _handle, data }
    }

    fn is_cache_valid(&self, size: Vec2) -> bool {
        match self.size_cache {
            None => false,
            Some(ref last) => last.x.accept(size.x) && last.y.accept(size.y),
        }
    }

    fn get_cache(&self) -> &InnerContentType {
        &self.content_cache
    }
}

/// A simple view showing a fixed text.
///
/// # Examples
///
/// ```rust
/// # use cursive_core::Cursive;
/// # use cursive_core::views::TextView;
/// let mut siv = Cursive::new();
///
/// siv.add_layer(TextView::new("Hello world!"));
/// ```
pub struct TextView {
    // content: String,
    content: TextContent,
    rows: Arc<Vec<Row>>,
    row_cache: LruCache<usize, Arc<Vec<Row>>>,

    align: Align,

    style: StyleType,

    // True if we can wrap long lines.
    wrap: bool,

    // ScrollBase make many scrolling-related things easier
    width: Option<usize>,
}

impl TextView {
    /// Creates a new TextView with the given content.
    pub fn new<S>(content: S) -> Self
        where
            S: Into<StyledString>,
    {
        Self::new_with_content(TextContent::new(content))
    }

    /// Creates a new TextView using the given `TextContent`.
    ///
    /// If you kept a clone of the given content, you'll be able to update it
    /// remotely.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use cursive_core::views::{TextView, TextContent};
    /// let mut content = TextContent::new("content");
    /// let view = TextView::new_with_content(content.clone());
    ///
    /// // Later, possibly in a different thread
    /// content.set_content("new content");
    /// assert!(view.get_content().source().contains("new"));
    /// ```
    pub fn new_with_content(content: TextContent) -> Self {
        log("new");

        TextView {
            row_cache: LruCache::new(NonZeroUsize::new(10).unwrap()),
            rows: Arc::new(Vec::new()),
            content,
            style: StyleType::default(),
            wrap: true,
            align: Align::top_left(),
            width: None,
        }
    }

    /// Creates a new empty `TextView`.
    pub fn empty() -> Self {
        TextView::new("")
    }


    /// Sets the style for the content.
    pub fn set_style<S: Into<StyleType>>(&mut self, style: S) {
        self.style = style.into();
    }


    /// Sets the style for the entire content.
    ///
    /// Chainable variant.

    pub fn style<S: Into<StyleType>>(self, style: S) -> Self {
        self.with(|s| s.set_style(style))
    }

    /// Disables content wrap for this view.
    ///
    /// This may be useful if you want horizontal scrolling.

    pub fn no_wrap(self) -> Self {
        self.with(|s| s.set_content_wrap(false))
    }

    /// Controls content wrap for this view.
    ///
    /// If `true` (the default), text will wrap long lines when needed.
    pub fn set_content_wrap(&mut self, wrap: bool) {
        self.wrap = wrap;
    }

    /// Sets the horizontal alignment for this view.

    pub fn h_align(mut self, h: HAlign) -> Self {
        self.align.h = h;

        self
    }

    /// Sets the vertical alignment for this view.

    pub fn v_align(mut self, v: VAlign) -> Self {
        self.align.v = v;

        self
    }

    /// Sets the alignment for this view.

    pub fn align(mut self, a: Align) -> Self {
        self.align = a;

        self
    }

    /// Center the text horizontally and vertically inside the view.

    pub fn center(mut self) -> Self {
        self.align = Align::center();
        self
    }

    /// Replace the text in this view.
    ///
    /// Chainable variant.

    pub fn content<S>(self, content: S) -> Self
        where
            S: Into<StyledString>,
    {
        self.with(|s| s.set_content(content))
    }

    /// Replace the text in this view.
    pub fn set_content<S>(&mut self, content: S)
        where
            S: Into<StyledString>,
    {
        self.content.set_content(content);
    }

    /// Append `content` to the end of a `TextView`.
    pub fn append<S>(&mut self, content: S)
        where
            S: Into<StyledString>,
    {
        self.content.append(content);
    }

    /// Returns the current text in this view.
    pub fn get_content(&self) -> TextContentRef {
        TextContentInner::get_content(&self.content.content)
    }

    /// Returns a shared reference to the content, allowing content mutation.
    pub fn get_shared_content(&mut self) -> TextContent {
        // We take &mut here without really needing it,
        // because it sort of "makes sense".
        TextContent {
            content: Arc::clone(&self.content.content),
        }
    }

    // This must be non-destructive, as it may be called
    // multiple times during layout.
    fn compute_rows(&mut self, size: Vec2) {
        log(format!("compute_rows({:?})", size));
        let size = if self.wrap { size } else { Vec2::max_value() };

        let mut content = self.content.content.lock().unwrap();
        if content.is_cache_valid(size) {
            return;
        }

        // Completely bust the cache
        // Just in case we fail, we don't want to leave a bad cache.
        content.size_cache = None;
        content.content_cache = Arc::clone(&content.content_value);

        if size.x == 0 {
            // Nothing we can do at this point.
            return;
        }

        if let Some(rows) = self.row_cache.get(&size.x) {
            log("!! cache hit");
            self.rows = Arc::clone(rows);
        } else {
            log("!! cache miss");
            self.rows = Arc::new(
                LinesIterator::new(content.get_cache().as_ref(), size.x).collect(),
            );
            self.row_cache.put(size.x, Arc::clone(&self.rows));
        }

        // Desired width
        self.width = if self.rows.iter().any(|row| row.is_wrapped) {
            // If any rows are wrapped, then require the full width.
            Some(size.x)
        } else {
            self.rows.iter().map(|row| row.width).max()
        }
    }
}

impl View for TextView {
    fn draw(&self, printer: &Printer) {
        log("draw");
        let h = self.rows.len();
        // If the content is smaller than the view, align it somewhere.
        let offset = self.align.v.get_offset(h, printer.size.y);
        let printer = &printer.offset((0, offset));

        let content = self.content.content.lock().unwrap();

        printer.with_style(self.style, |printer| {
            for (y, row) in self
                .rows
                .iter()
                .enumerate()
                .skip(printer.content_offset.y)
                .take(printer.output_size.y)
            {
                let l = row.width;
                let mut x = self.align.h.get_offset(l, printer.size.x);

                for span in row.resolve_stream(content.get_cache().as_ref()) {
                    printer.with_style(*span.attr, |printer| {
                        printer.print((x, y), span.content);
                        x += span.content.width();
                    });
                }
            }
        });
    }

    fn layout(&mut self, size: Vec2) {
        log(format!("layout({:?})", size));
        // Compute the text rows.
        self.compute_rows(size);

        // The entire "virtual" size (includes all rows)
        let my_size = Vec2::new(self.width.unwrap_or(0), self.rows.len());

        // Build a fresh cache.
        let mut content = self.content.content.lock().unwrap();
        content.size_cache = Some(SizeCache::build(my_size, size));
    }

    fn needs_relayout(&self) -> bool {
        log("needs_relayout");
        let content = self.content.content.lock().unwrap();
        content.size_cache.is_none()
    }

    fn required_size(&mut self, size: Vec2) -> Vec2 {
        log(format!("required_size({:?})", size));
        self.compute_rows(size);

        Vec2::new(self.width.unwrap_or(0), self.rows.len())
    }
}
