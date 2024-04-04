//! This example shows how to use pdf-writer in a bit more realisitic
//! circumstances.
//!
//! It demonstrates how to allocate IDs on the fly using [`Ref::bump`] and
//! shows how to write multiple things at once using [`Chunk`]s.

use pdf_writer::{Chunk, Content, Name, Pdf, Rect, Ref};

fn main() -> std::io::Result<()> {
    // Define an ID allocator. Every time we need a new object, we just call
    // `alloc.bump()`, which increases `alloc` by one and returns its previous
    // value.
    let mut alloc = Ref::new(1);

    // Start writing.
    let mut pdf = Pdf::new();

    // Create a secondary chunk for parallel writing. This will allows us to
    // write stuff while we're also holding a mutable reference to the main
    // writer.
    let mut secondary = Chunk::new();

    // We'll collect the page IDs here.
    let page_tree_id = alloc.bump();
    let mut page_ids = vec![];

    // Write a few pages with different colors.
    for p in 0..10 {
        // We'll create five shapes with varying transparency.
        let count = 5;

        // Write the page.
        let page_id = alloc.bump();
        page_ids.push(page_id);
        let mut page = pdf.page(page_id);
        let width = 90.0 + 60.0 * count as f32;
        page.media_box(Rect::new(0.0, 0.0, width, 200.0));
        page.parent(page_tree_id);

        // Create the page's content.
        let mut content = Content::new();
        let mut extg_states = vec![];
        for i in 1..=count {
            // Define a unique name for the ExtG state.
            let state_name = format!("G_{p}_{i}");

            // Write the content that references the state.
            let x = 60.0 * i as f32 - 10.0;
            content.set_parameters(Name(state_name.as_bytes()));
            content.set_fill_rgb(1.0, 0.1 * p as f32, 0.0);
            content.rect(x, 50.0, 50.0, 100.0);
            content.fill_even_odd();

            // Write the ExtG state into the secondary chunk. Note how we can
            // write into it even though the main `pdf` writer is currently
            // borrowed.
            //
            // In a real application, we could also try to deduplicate ExtG
            // states here to save precious file size.
            let state_id = alloc.bump();
            let alpha = i as f32 / count as f32;
            secondary.ext_graphics(state_id).non_stroking_alpha(alpha);
            extg_states.push((state_name, state_id));
        }

        // Write the content, add it to the secondary chunk (the main chunk is
        // still borrowed) and register it with the page.
        let content_id = alloc.bump();
        secondary.stream(content_id, &content.finish());
        page.contents(content_id);

        // Reference the ExtG states that we wrote in the page's resources
        // dictionary.
        page.resources()
            .ext_g_states()
            .pairs(extg_states.iter().map(|(n, id)| (Name(n.as_bytes()), id)));
    }

    // Add the ExtG states to the PDF.
    pdf.extend(&secondary);

    // Write the root of the page tree.
    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_ids.len() as i32);

    // Write the document catalog.
    pdf.catalog(alloc.bump()).pages(page_tree_id);

    // Finish and write the thing to a file.
    std::fs::write("target/chunks.pdf", pdf.finish())
}
