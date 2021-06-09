# pdf-writer
[![Build status](https://github.com/typst/pdf-writer/workflows/Continuous%20integration/badge.svg)](https://github.com/typst/pdf-writer/actions)
[![Crates.io](https://img.shields.io/crates/v/pdf-writer.svg)](https://crates.io/crates/pdf-writer)
[![Documentation](https://docs.rs/pdf-writer/badge.svg)](https://docs.rs/pdf-writer)

A step-by-step, zero-unsafe PDF writer.

The entry point into the API is the main `PdfWriter`, which constructs the
document into one big internal buffer. The top-level writer has many methods to
create specialized writers for specific PDF objects. These all follow the same
general pattern: They borrow the main buffer mutably, expose a builder pattern
for writing individual fields in a strongly typed fashion and finish up the
object when dropped.

## Minimal example
The following example creates a PDF with a single, empty A4 page.

```rust
use pdf_writer::{PdfWriter, Rect, Ref};

// Define some indirect reference ids we'll use.
let catalog_id = Ref::new(1);
let page_tree_id = Ref::new(2);
let page_id = Ref::new(3);

// Start writing with the PDF version 1.7 header.
let mut writer = PdfWriter::new(1, 7);

// The document catalog and a page tree with one A4 page that uses no resources.
writer.catalog(catalog_id).pages(page_tree_id);
writer.pages(page_tree_id).kids(vec![page_id]);
writer.page(page_id)
    .parent(page_tree_id)
    .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
    .resources();

// Finish with cross-reference table and trailer and write to file.
std::fs::write("target/empty.pdf", writer.finish(catalog_id))?;
```

For a more comprehensive overview, check out the [hello world example] in the
repository, which creates a document with text in it.

## License
This crate is dual-licensed under the MIT and Apache 2.0 licenses.

[hello world example]: https://github.com/typst/pdf-writer/tree/main/examples/hello.rs
