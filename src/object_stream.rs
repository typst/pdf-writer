use std::io::Write;

use crate::chunk::ChunkEntry;
use crate::{Chunk, Filter, Limits, Name, Ref, Settings};

#[derive(Clone)]
struct PackedObject {
    id: Ref,
    body: Vec<u8>,
}

/// The filters used for an object stream.
pub enum ObjectStreamFilter {
    /// A single filter.
    Single(Filter),
    /// An array of filters.
    Multiple(Vec<Filter>),
}

/// A batch of objects ready to be serialized as one object stream.
pub struct ObjectStreamJob {
    object_stream_ref: Ref,
    objects: Vec<PackedObject>,
    settings: Settings,
    source_limits: Limits,
}

impl ObjectStreamJob {
    /// Build the object stream chunk without applying a filter.
    pub fn build(self) -> Chunk {
        self.build_inner(|data| (data, None))
    }

    /// Build the object stream chunk and apply a filter to the stream data.
    pub fn build_with_filter(
        self,
        filter: impl FnOnce(&[u8]) -> (Vec<u8>, ObjectStreamFilter),
    ) -> Chunk {
        self.build_inner(|data| {
            let (filtered, filter) = filter(&data);
            (filtered, Some(filter))
        })
    }

    fn build_inner(
        self,
        filter: impl FnOnce(Vec<u8>) -> (Vec<u8>, Option<ObjectStreamFilter>),
    ) -> Chunk {
        let mut offsets = Vec::new();
        let mut bodies = Vec::new();

        for object in &self.objects {
            offsets.push((object.id, bodies.len()));
            bodies.extend_from_slice(&object.body);
            bodies.push(b'\n');
        }

        let mut data = Vec::new();
        for (id, offset) in &offsets {
            write!(&mut data, "{} {} ", id.get(), offset).unwrap();
        }

        let first = data.len();
        data.extend_from_slice(&bodies);

        let (data, filter) = filter(data);

        let mut chunk = Chunk::with_settings(self.settings);
        chunk.merge_limits(&self.source_limits);
        {
            let mut stream = chunk.stream(self.object_stream_ref, &data);
            stream
                .pair(Name(b"Type"), Name(b"ObjStm"))
                .pair(Name(b"N"), self.objects.len() as i32)
                .pair(Name(b"First"), first as i32);

            if let Some(filter) = filter {
                match filter {
                    ObjectStreamFilter::Single(filter) => {
                        stream.filter(filter);
                    }
                    ObjectStreamFilter::Multiple(filters) => {
                        let mut arr = stream.insert(Name(b"Filter")).array();

                        for filter in filters {
                            arr.item(filter.to_name());
                        }
                    }
                }
            }
        }

        for (index, object) in self.objects.into_iter().enumerate() {
            chunk.entries.push(ChunkEntry::Compressed {
                id: object.id,
                object_stream: self.object_stream_ref,
                index: index as u32,
            });
        }

        chunk
    }
}

/// Builds object stream jobs from chunks containing ordinary indirect objects.
pub struct ObjectStreamBuilder<T> {
    settings: Settings,
    max_objects: usize,
    pending: Vec<PackedObject>,
    pending_limits: Limits,
    streams: Vec<T>,
}

impl<T> ObjectStreamBuilder<T> {
    /// Create a new object stream builder.
    ///
    /// `max_objects` controls how many objects are written into each object
    /// stream and must be greater than zero.
    pub fn new(settings: Settings, max_objects: usize) -> Self {
        assert!(max_objects > 0, "object streams must contain at least one object");
        Self {
            settings,
            max_objects,
            pending: Vec::new(),
            pending_limits: Limits::new(),
            streams: Vec::new(),
        }
    }

    /// Extend the builder with all objects from `chunk`.
    ///
    /// Panics if the chunk contains stream objects.
    pub fn extend_with(
        &mut self,
        chunk: &Chunk,
        next_ref: &mut impl FnMut() -> Ref,
        spawn: &mut impl FnMut(ObjectStreamJob) -> T,
    ) {
        assert!(
            !chunk.has_streams(),
            "object streams can only contain non-stream objects"
        );

        for object in chunk.indirect_objects() {
            // Objects written through the public API always have generation 0.
            assert_eq!(
                object.generation, 0,
                "object streams can only contain generation 0 objects"
            );

            self.pending_limits.merge(chunk.limits());
            self.pending
                .push(PackedObject { id: object.id, body: object.body.to_vec() });

            if self.pending.len() == self.max_objects {
                self.flush(next_ref, spawn);
            }
        }
    }

    /// Finish the builder and return all spawned object streams.
    pub fn finish_with(
        mut self,
        next_ref: &mut impl FnMut() -> Ref,
        spawn: &mut impl FnMut(ObjectStreamJob) -> T,
    ) -> Vec<T> {
        self.flush(next_ref, spawn);
        self.streams
    }

    fn flush(
        &mut self,
        next_ref: &mut impl FnMut() -> Ref,
        spawn: &mut impl FnMut(ObjectStreamJob) -> T,
    ) {
        if self.pending.is_empty() {
            return;
        }

        let job = ObjectStreamJob {
            object_stream_ref: next_ref(),
            objects: std::mem::take(&mut self.pending),
            settings: self.settings,
            source_limits: std::mem::replace(&mut self.pending_limits, Limits::new()),
        };
        self.streams.push(spawn(job));
    }
}
