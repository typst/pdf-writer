use super::*;
use crate::buf::Buf;

/// A builder for a collection of indirect PDF objects.
///
/// This type holds written top-level indirect PDF objects. Typically, you won't
/// create a colllection yourself, but use the primary chunk of the top-level
/// [`Pdf`] through its [`Deref`] implementation.
///
/// However, sometimes it's useful to be able to create a separate chunk to be
/// able to write two things at the same time (which isn't possible with a
/// single chunk because of the streaming nature --- only one writer can borrow
/// it at a time).
#[derive(Clone)]
pub struct Chunk {
    pub(crate) buf: Buf,
    pub(crate) offsets: Vec<(Ref, usize)>,
}

impl Chunk {
    /// Create a new chunk with the default capacity (currently 1 KB).
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new chunk with the specified initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: Buf::with_capacity(capacity), offsets: vec![] }
    }

    /// The number of bytes that were written so far.
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// The bytes already written so far.
    pub fn as_bytes(&self) -> &[u8] {
        self.buf.deref()
    }

    /// Return the limits of the chunk.
    pub fn limits(&self) -> &Limits {
        self.buf.limits()
    }

    /// Add all objects from another chunk to this one.
    pub fn extend(&mut self, other: &Chunk) {
        let base = self.len();
        self.buf.extend_buf(&other.buf);
        self.offsets
            .extend(other.offsets.iter().map(|&(id, offset)| (id, base + offset)));
    }

    /// An iterator over the references of the top-level objects
    /// of the chunk, in the order they appear in the chunk.
    pub fn refs(&self) -> impl ExactSizeIterator<Item = Ref> + '_ {
        self.offsets.iter().map(|&(id, _)| id)
    }

    /// Renumbers the IDs of indirect objects and all indirect references in the
    /// chunk and returns the resulting chunk.
    ///
    /// The given closure is called for each object and indirect reference in
    /// the chunk. When an ID appears multiple times in the chunk (for object
    /// and/or reference), it will be called multiple times. When assigning new
    /// IDs, it is up to you to provide a well-defined mapping (it should most
    /// probably be a pure function so that a specific old ID is always mapped
    /// to the same new ID).
    ///
    /// A simple way to renumber a chunk is to map all old IDs to new
    /// consecutive IDs. This can be achieved by allocating a new ID for each
    /// unique ID we have seen and memoizing this mapping in a hash map:
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use pdf_writer::{Chunk, Ref, TextStr, Name};
    /// let mut chunk = Chunk::new();
    /// chunk.indirect(Ref::new(10)).primitive(true);
    /// chunk.indirect(Ref::new(17))
    ///     .dict()
    ///     .pair(Name(b"Self"), Ref::new(17))
    ///     .pair(Name(b"Ref"), Ref::new(10))
    ///     .pair(Name(b"NoRef"), TextStr("Text with 10 0 R"));
    ///
    /// // Gives the objects consecutive IDs.
    /// // - The `true` object will get ID 1.
    /// // - The dictionary object will get ID 2.
    /// let mut alloc = Ref::new(1);
    /// let mut map = HashMap::new();
    /// let renumbered = chunk.renumber(|old| {
    ///     *map.entry(old).or_insert_with(|| alloc.bump())
    /// });
    /// ```
    ///
    /// If a chunk references indirect objects that are not defined within it,
    /// the closure is still called with those references. Allocating new IDs
    /// for them will probably not make sense, so it's up to you to either not
    /// have dangling references or handle them in a way that makes sense for
    /// your use case.
    pub fn renumber<F>(&self, mapping: F) -> Chunk
    where
        F: FnMut(Ref) -> Ref,
    {
        let mut chunk = Chunk::with_capacity(self.len());
        self.renumber_into(&mut chunk, mapping);
        chunk
    }

    /// Same as [`renumber`](Self::renumber), but writes the results into an
    /// existing `target` chunk instead of creating a new chunk.
    pub fn renumber_into<F>(&self, target: &mut Chunk, mut mapping: F)
    where
        F: FnMut(Ref) -> Ref,
    {
        target.buf.reserve(self.len());
        crate::renumber::renumber(self, target, &mut mapping);
    }
}

/// Indirect objects and streams.
impl Chunk {
    /// Start writing an indirectly referenceable object.
    pub fn indirect(&mut self, id: Ref) -> Obj<'_> {
        self.offsets.push((id, self.buf.len()));
        Obj::indirect(&mut self.buf, id)
    }

    /// Start writing an indirectly referenceable stream.
    ///
    /// The stream data and the `/Length` field are written automatically. You
    /// can add additional key-value pairs to the stream dictionary with the
    /// returned stream writer.
    ///
    /// You can use this function together with a [`Content`] stream builder to
    /// provide a [page's contents](Page::contents).
    /// ```
    /// use pdf_writer::{Pdf, Content, Ref};
    ///
    /// // Create a simple content stream.
    /// let mut content = Content::new();
    /// content.rect(50.0, 50.0, 50.0, 50.0);
    /// content.stroke();
    ///
    /// // Create a writer and write the stream.
    /// let mut pdf = Pdf::new();
    /// pdf.stream(Ref::new(1), &content.finish());
    /// ```
    ///
    /// This crate does not do any compression for you. If you want to compress
    /// a stream, you have to pass already compressed data into this function
    /// and specify the appropriate filter in the stream dictionary.
    ///
    /// For example, if you want to compress your content stream with DEFLATE,
    /// you could do something like this:
    /// ```
    /// use pdf_writer::{Pdf, Content, Ref, Filter};
    /// use miniz_oxide::deflate::{compress_to_vec_zlib, CompressionLevel};
    ///
    /// // Create a simple content stream.
    /// let mut content = Content::new();
    /// content.rect(50.0, 50.0, 50.0, 50.0);
    /// content.stroke();
    ///
    /// // Compress the stream.
    /// let level = CompressionLevel::DefaultLevel as u8;
    /// let compressed = compress_to_vec_zlib(&content.finish(), level);
    ///
    /// // Create a writer, write the compressed stream and specify that it
    /// // needs to be decoded with a FLATE filter.
    /// let mut pdf = Pdf::new();
    /// pdf.stream(Ref::new(1), &compressed).filter(Filter::FlateDecode);
    /// ```
    /// For all the specialized stream functions below, it works the same way:
    /// You can pass compressed data and specify a filter.
    ///
    /// Panics if the stream length exceeds `i32::MAX`.
    pub fn stream<'a>(&'a mut self, id: Ref, data: &'a [u8]) -> Stream<'a> {
        Stream::start(self.indirect(id), data)
    }
}

/// Document structure.
impl Chunk {
    /// Start writing a page tree.
    pub fn pages(&mut self, id: Ref) -> Pages<'_> {
        self.indirect(id).start()
    }

    /// Start writing a page.
    pub fn page(&mut self, id: Ref) -> Page<'_> {
        self.indirect(id).start()
    }

    /// Start writing an outline.
    pub fn outline(&mut self, id: Ref) -> Outline<'_> {
        self.indirect(id).start()
    }

    /// Start writing an outline item.
    pub fn outline_item(&mut self, id: Ref) -> OutlineItem<'_> {
        self.indirect(id).start()
    }

    /// Start writing a destination for use in a name tree.
    pub fn destination(&mut self, id: Ref) -> Destination<'_> {
        self.indirect(id).start()
    }

    /// Start writing a named destination dictionary.
    pub fn destinations(&mut self, id: Ref) -> TypedDict<'_, Destination> {
        self.indirect(id).dict().typed()
    }

    /// Start writing a file specification dictionary.
    pub fn file_spec(&mut self, id: Ref) -> FileSpec<'_> {
        self.indirect(id).start()
    }

    /// Start writing an embedded file stream.
    pub fn embedded_file<'a>(&'a mut self, id: Ref, bytes: &'a [u8]) -> EmbeddedFile<'a> {
        EmbeddedFile::start(self.stream(id, bytes))
    }

    /// Start writing a structure tree element.
    pub fn struct_element(&mut self, id: Ref) -> StructElement<'_> {
        self.indirect(id).start()
    }

    /// Start writing a metadata stream.
    pub fn metadata<'a>(&'a mut self, id: Ref, bytes: &'a [u8]) -> Metadata<'a> {
        Metadata::start(self.stream(id, bytes))
    }
}

/// Graphics and content.
impl Chunk {
    /// Start writing an image XObject stream.
    ///
    /// The samples should be encoded according to the stream's filter, color
    /// space and bits per component.
    pub fn image_xobject<'a>(
        &'a mut self,
        id: Ref,
        samples: &'a [u8],
    ) -> ImageXObject<'a> {
        ImageXObject::start(self.stream(id, samples))
    }

    /// Start writing a form XObject stream.
    ///
    /// These can be used as transparency groups.
    ///
    /// Note that these have nothing to do with forms that have fields to fill
    /// out. Rather, they are a way to encapsulate and reuse content across the
    /// file.
    ///
    /// You can create the content bytes using a [`Content`] builder.
    pub fn form_xobject<'a>(&'a mut self, id: Ref, content: &'a [u8]) -> FormXObject<'a> {
        FormXObject::start(self.stream(id, content))
    }

    /// Start writing an external graphics state dictionary.
    pub fn ext_graphics(&mut self, id: Ref) -> ExtGraphicsState<'_> {
        self.indirect(id).start()
    }
}

/// Fonts.
impl Chunk {
    /// Start writing a Type-1 font.
    pub fn type1_font(&mut self, id: Ref) -> Type1Font<'_> {
        self.indirect(id).start()
    }

    /// Start writing a Type-3 font.
    pub fn type3_font(&mut self, id: Ref) -> Type3Font<'_> {
        self.indirect(id).start()
    }

    /// Start writing a Type-0 font.
    pub fn type0_font(&mut self, id: Ref) -> Type0Font<'_> {
        self.indirect(id).start()
    }

    /// Start writing a CID font.
    pub fn cid_font(&mut self, id: Ref) -> CidFont<'_> {
        self.indirect(id).start()
    }

    /// Start writing a font descriptor.
    pub fn font_descriptor(&mut self, id: Ref) -> FontDescriptor<'_> {
        self.indirect(id).start()
    }

    /// Start writing a character map stream.
    ///
    /// If you want to use this for a `/ToUnicode` CMap, you can create the
    /// bytes using a [`UnicodeCmap`](types::UnicodeCmap) builder.
    pub fn cmap<'a>(&'a mut self, id: Ref, cmap: &'a [u8]) -> Cmap<'a> {
        Cmap::start(self.stream(id, cmap))
    }
}

/// Color spaces, shadings and patterns.
impl Chunk {
    /// Start writing a color space.
    pub fn color_space(&mut self, id: Ref) -> ColorSpace<'_> {
        self.indirect(id).start()
    }

    /// Start writing a function-based shading (type 1-3).
    pub fn function_shading(&mut self, id: Ref) -> FunctionShading<'_> {
        self.indirect(id).start()
    }

    /// Start writing a stream-based shading (type 4-7).
    pub fn stream_shading<'a>(
        &'a mut self,
        id: Ref,
        content: &'a [u8],
    ) -> StreamShading<'a> {
        StreamShading::start(self.stream(id, content))
    }

    /// Start writing a tiling pattern stream.
    ///
    /// You can create the content bytes using a [`Content`] builder.
    pub fn tiling_pattern<'a>(
        &'a mut self,
        id: Ref,
        content: &'a [u8],
    ) -> TilingPattern<'a> {
        TilingPattern::start_with_stream(self.stream(id, content))
    }

    /// Start writing a shading pattern.
    pub fn shading_pattern(&mut self, id: Ref) -> ShadingPattern<'_> {
        self.indirect(id).start()
    }

    /// Start writing an ICC profile stream.
    ///
    /// The `profile` argument shall contain the ICC profile data conforming to
    /// ICC.1:2004-10 (PDF 1.7), ICC.1:2003-09 (PDF 1.6), ICC.1:2001-12 (PDF 1.5),
    /// ICC.1:1999-04 (PDF 1.4), or ICC 3.3 (PDF 1.3). Profile data is commonly
    /// compressed using the `FlateDecode` filter.
    pub fn icc_profile<'a>(&'a mut self, id: Ref, profile: &'a [u8]) -> IccProfile<'a> {
        IccProfile::start(self.stream(id, profile))
    }
}

/// Functions.
impl Chunk {
    /// Start writing a sampled function stream.
    pub fn sampled_function<'a>(
        &'a mut self,
        id: Ref,
        samples: &'a [u8],
    ) -> SampledFunction<'a> {
        SampledFunction::start(self.stream(id, samples))
    }

    /// Start writing an exponential function.
    pub fn exponential_function(&mut self, id: Ref) -> ExponentialFunction<'_> {
        self.indirect(id).start()
    }

    /// Start writing a stitching function.
    pub fn stitching_function(&mut self, id: Ref) -> StitchingFunction<'_> {
        self.indirect(id).start()
    }

    /// Start writing a PostScript function stream.
    ///
    /// You can create the code bytes using [`PostScriptOp::encode`](types::PostScriptOp::encode).
    pub fn post_script_function<'a>(
        &'a mut self,
        id: Ref,
        code: &'a [u8],
    ) -> PostScriptFunction<'a> {
        PostScriptFunction::start(self.stream(id, code))
    }
}

/// Tree data structures.
impl Chunk {
    /// Start writing a name tree node.
    pub fn name_tree<T: Primitive>(&mut self, id: Ref) -> NameTree<'_, T> {
        self.indirect(id).start()
    }

    /// Start writing a number tree node.
    pub fn number_tree<T: Primitive>(&mut self, id: Ref) -> NumberTree<'_, T> {
        self.indirect(id).start()
    }
}

/// Interactive features.
impl Chunk {
    /// Start writing an annotation dictionary.
    pub fn annotation(&mut self, id: Ref) -> Annotation<'_> {
        self.indirect(id).start()
    }

    /// Start writing a form field dictionary.
    pub fn form_field(&mut self, id: Ref) -> Field<'_> {
        self.indirect(id).start()
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad("Chunk(..)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk() {
        let mut w = Pdf::new();
        let mut font = w.type3_font(Ref::new(1));
        let mut c = Chunk::new();
        c.font_descriptor(Ref::new(2)).name(Name(b"MyFont"));
        font.font_descriptor(Ref::new(2));
        font.finish();
        w.extend(&c);
        test!(
            w.finish(),
            b"%PDF-1.7\n%\x80\x80\x80\x80\n",
            b"1 0 obj",
            b"<<\n  /Type /Font\n  /Subtype /Type3\n  /FontDescriptor 2 0 R\n>>",
            b"endobj\n",
            b"2 0 obj",
            b"<<\n  /Type /FontDescriptor\n  /FontName /MyFont\n>>",
            b"endobj\n",
            b"xref",
            b"0 3",
            b"0000000000 65535 f\r",
            b"0000000016 00000 n\r",
            b"0000000094 00000 n\r",
            b"trailer",
            b"<<\n  /Size 3\n>>",
            b"startxref\n160\n%%EOF",
        );
    }
}
