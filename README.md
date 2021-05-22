# pdf-writer
[![Build status](https://github.com/typst/pdf-writer/workflows/Continuous%20integration/badge.svg)](https://github.com/typst/pdf-writer/actions)
[![Crates.io](https://img.shields.io/crates/v/pdf-writer.svg)](https://crates.io/crates/pdf-writer)
[![Documentation](https://docs.rs/pdf-writer/badge.svg)](https://docs.rs/pdf-writer)

A step-by-step PDF writer.

The entry point into the API is the main `PdfWriter`. The document is written
into an internal buffer, but otherwise the API is largely non-allocating.

## Minimal example
The following example creates a PDF with a single, empty A4 page.

```rust
use pdf_writer::{PdfWriter, Rect, Ref};

// Start writing with the PDF version 1.7 header.
let mut writer = PdfWriter::new(1, 7);

// The document catalog and a page tree with one A4 page that uses no resources.
writer.catalog(Ref::new(1)).pages(Ref::new(2));
writer.pages(Ref::new(2)).kids(vec![Ref::new(3)]);
writer.page(Ref::new(3))
    .parent(Ref::new(2))
    .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
    .resources();

// Finish with cross-reference table and trailer and write to file.
std::fs::write("empty.pdf", writer.finish(Ref::new(1)))?;
```

For a more comprehensive overview, check out the [hello world example] in the
repository, which creates a document with text in it.

## License
This crate is dual-licensed under the MIT and Apache 2.0 licenses.

[hello world example]: https://github.com/typst/pdf-writer/tree/main/examples/hello.rs
