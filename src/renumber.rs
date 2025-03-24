use crate::{Buf, Chunk, Ref};

/// Renumbers a chunk of objects.
///
/// See [`Chunk::renumber`] for more details.
pub fn renumber(source: &Chunk, target: &mut Chunk, mapping: &mut dyn FnMut(Ref) -> Ref) {
    target.buf.limits.merge(source.limits());

    let mut iter = source.offsets.iter().copied().peekable();
    while let Some((id, offset)) = iter.next() {
        let new = mapping(id);
        let end = iter.peek().map_or(source.buf.len(), |&(_, offset)| offset);
        let slice = &source.buf[offset..end];
        let Some((gen, slice)) = extract_object(slice) else { continue };
        target.offsets.push((new, target.buf.len()));
        target.buf.push_int(new.get());
        target.buf.push(b' ');
        target.buf.push_int(gen);
        target.buf.extend(b" obj\n");
        patch_object(slice, &mut target.buf, mapping);
        target.buf.extend(b"\nendobj\n\n");
    }
}

/// Extract the generation number and interior of an indirect object.
fn extract_object(slice: &[u8]) -> Option<(i32, &[u8])> {
    let offset = memchr::memmem::find(slice, b"obj")?;
    let mut prefix = &slice[..offset];
    require_whitespace_rev(&mut prefix);
    let gen = eat_number_rev(&mut prefix)?;

    let mut head = offset + 3;
    while slice.get(head).copied().map_or(false, is_whitespace) {
        head += 1;
    }

    let mut tail = memchr::memmem::rfind(slice, b"endobj")?;
    while tail > 0 && slice.get(tail - 1).copied().map_or(false, is_whitespace) {
        tail -= 1;
    }

    let data = slice.get(head..tail)?;
    Some((gen, data))
}

/// Processes the interior of an indirect object and patches all indirect
/// references.
fn patch_object(slice: &[u8], buf: &mut Buf, mapping: &mut dyn FnMut(Ref) -> Ref) {
    // Find the next point of interest:
    // - 'R' is interesting because it could be an indirect reference
    // - Anything that could contain indirect-reference-like things that are not
    //   actually indirect references is interesting
    //   - 's' could start a stream
    //   - '(' starts a string
    //   - Names are not a problem because they can't contain literal whitespace
    //   - Hexadecimal strings are not a problem because they can't contain R
    //   - There are no other collection of arbitrary bytes
    let mut written = 0;
    let mut seen = 0;
    while seen < slice.len() {
        match slice[seen] {
            // Validate whether this is an indirect reference and if it is,
            // patch it!
            b'R' => {
                if let Some((head, id, gen)) = validate_ref(&slice[..seen]) {
                    let new = mapping(id);
                    buf.extend(&slice[written..head]);
                    buf.push_int(new.get());
                    buf.push(b' ');
                    buf.push_int(gen);
                    buf.push(b' ');
                    buf.push(b'R');
                    written = seen + 1;
                }
            }

            // Skip comments.
            b'%' => {
                while seen < slice.len() {
                    match slice[seen] {
                        b'\n' | b'\r' => break,
                        _ => {}
                    }
                    seen += 1;
                }
            }

            // Skip strings.
            b'(' => {
                let mut depth = 0;
                while seen < slice.len() {
                    match slice[seen] {
                        b'(' => depth += 1,
                        b')' if depth == 1 => break,
                        b')' => depth -= 1,
                        b'\\' => seen += 1,
                        _ => {}
                    }
                    seen += 1;
                }
            }

            // Check whether this is the start of a stream. If yes, we can bail
            // and copy the rest verbatim.
            b's' if slice[seen..].starts_with(b"stream")
                && validate_stream(&slice[..seen]) =>
            {
                break;
            }

            _ => {}
        }

        seen += 1;
    }

    buf.extend(&slice[written..]);
}

/// Validate a match for an indirect reference.
fn validate_ref(mut prefix: &[u8]) -> Option<(usize, Ref, i32)> {
    require_whitespace_rev(&mut prefix)?;
    let gen = eat_number_rev(&mut prefix)?;
    require_whitespace_rev(&mut prefix)?;
    let id = eat_number_rev(&mut prefix)?;
    (id > 0).then(|| (prefix.len(), Ref::new(id), gen))
}

/// Validate a match for a stream.
fn validate_stream(mut prefix: &[u8]) -> bool {
    eat_suffix(&mut prefix, is_whitespace);
    prefix.ends_with(b">>")
}

/// Require at least one byte of whitespace, in reverse.
fn require_whitespace_rev(slice: &mut &[u8]) -> Option<()> {
    (!eat_suffix(slice, is_whitespace).is_empty()).then_some(())
}

/// Eat an ASCII number, in reverse.
fn eat_number_rev(slice: &mut &[u8]) -> Option<i32> {
    let tail = eat_suffix(slice, |byte| byte.is_ascii_digit());
    let string = std::str::from_utf8(tail).ok()?;
    string.parse::<i32>().ok()
}

/// Eat a suffix that fulfills a predicate.
fn eat_suffix<'a>(slice: &mut &'a [u8], predicate: fn(u8) -> bool) -> &'a [u8] {
    let mut i = slice.len();
    while i > 0 && predicate(slice[i - 1]) {
        i -= 1;
    }
    let (head, tail) = slice.split_at(i);
    *slice = head;
    tail
}

/// Whether a character is whitespace according to PDF syntax conventions.
fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b'\0' | b'\t' | b'\n' | b'\x0c' | b'\r' | b' ')
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{Name, Ref, TextStr};

    #[test]
    fn test_renumber() {
        let mut c = Chunk::new();

        c.indirect(Ref::new(4))
            .dict()
            .insert(Name(b"Nested"))
            .dict()
            .pair(Name(b"stream"), true)
            .pair(Name(b"Nonself"), Ref::new(8))
            .pair(Name(b"R"), false)
            .pair(Name(b"Safe"), 10)
            .pair(Name(b"False"), TextStr("17 0 R"))
            .pair(Name(b"Unsafe"), Ref::new(17))
            .pair(Name(b"Unsafe"), TextStr("()(And (More) 17 0 R())"));

        // Manually write an untidy object.
        c.offsets.push((Ref::new(8), c.buf.len()));
        c.buf.extend(b"8  3  obj\n<</Fmt false/Niceness(4 0\nR-)");
        c.buf.extend(b"/beginobj/endobj%4 0 R\n");
        c.buf.extend(b"/Me 8 3  R/Unknown 11 0  R/R[4  0\nR]>>%\n\nendobj");

        c.stream(Ref::new(17), b"1 0 R 2 0 R 3 0 R 4 0 R")
            .pair(Name(b"Ok"), TextStr(")4 0 R"))
            .pair(Name(b"Self"), Ref::new(17));

        let mut alloc = Ref::new(1);
        let mut map = HashMap::new();
        let r = c.renumber(|old| {
            if old == Ref::new(11) {
                old
            } else {
                *map.entry(old).or_insert_with(|| alloc.bump())
            }
        });

        test!(
            r.buf.as_slice(),
            b"1 0 obj",
            b"<<",
            b"  /Nested <<",
            b"    /stream true",
            b"    /Nonself 2 0 R",
            b"    /R false",
            b"    /Safe 10",
            b"    /False (17 0 R)",
            b"    /Unsafe 3 0 R",
            b"    /Unsafe (()(And (More) 17 0 R()))",
            b"  >>",
            b">>",
            b"endobj",
            b"",
            b"2 3 obj",
            b"<</Fmt false/Niceness(4 0",
            b"R-)/beginobj/endobj%4 0 R",
            b"/Me 2 3 R/Unknown 11 0 R/R[1 0 R]>>%",
            b"endobj",
            b"",
            b"3 0 obj",
            b"<<",
            b"  /Length 23",
            b"  /Ok (\\)4 0 R)",
            b"  /Self 3 0 R",
            b">>",
            b"stream",
            b"1 0 R 2 0 R 3 0 R 4 0 R",
            b"endstream",
            b"endobj\n\n"
        );
    }
}
