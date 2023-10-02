# pdf-writer
[![Crates.io](https://img.shields.io/crates/v/pdf-writer.svg)](https://crates.io/crates/pdf-writer)
[![Documentation](https://docs.rs/pdf-writer/badge.svg)](https://docs.rs/pdf-writer)

A step-by-step PDF writer.

```toml
[dependencies]
pdf-writer = "0.8"
```

The entry point into the API is the main `Pdf`, which constructs the document
into one big internal buffer. The top-level writer has many methods to create
specialized writers for specific PDF objects. These all follow the same general
pattern: They borrow the main buffer mutably, expose a builder pattern for
writing individual fields in a strongly typed fashion and finish up the object
when dropped.

There are a few more top-level structs with internal buffers, like the builder
for `Content` streams, but wherever possible buffers are borrowed from parent
writers to minimize allocations.

## Minimal example
The following example creates a PDF with a single, empty A4 page.

```rust
use pdf_writer::{Pdf, Rect, Ref};

// Define some indirect reference ids we'll use.
let catalog_id = Ref::new(1);
let page_tree_id = Ref::new(2);
let page_id = Ref::new(3);

// Write a document catalog and a page tree with one A4 page that uses no resources.
let mut pdf = Pdf::new();
pdf.catalog(catalog_id).pages(page_tree_id);
pdf.pages(page_tree_id).kids([page_id]).count(1);
pdf.page(page_id)
    .parent(page_tree_id)
    .media_box(Rect::new(0.0, 0.0, 595.0, 842.0))
    .resources();

// Finish with cross-reference table and trailer and write to file.
std::fs::write("target/empty.pdf", pdf.finish())?;
```

For more examples, check out the [examples folder] in the repository.

## Safety
This crate forbids unsafe code, but it depends on a few popular crates that use
unsafe internally.

## License
This crate is dual-licensed under the MIT and Apache 2.0 licenses.

[examples folder]: https://github.com/typst/pdf-writer/tree/main/examples
