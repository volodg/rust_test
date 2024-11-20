use crate::json_stream_parser::{JsonEvent, JsonStreamParser};

mod json_stream_parser;

// TODOs:
// 1. L1 Cache-Friendly Buffer <= 32 kB
// 2. Memory aligning using #[repr(align(64))]
// #[repr(align(64))]
// struct AlignedBuffer([u8; L1_CACHE_SIZE]);
// 3. thread affinity
// 4. Using simd to handle packets, not a byte per byte
// 5. Use perf/cachegrind tool to find cache miss, etc
// 6. TODO try nom library instead?
// 7. Is not complete Json standard implementation

// TODO:
// 1. measure performance
// 2. create producer/consumer
// 3. Generate test data
// 4. play with initial vector sizes "Vec::<u8>::with_capacity(1024); // 1k"

fn main() {
    let json = r#"
        {
            "name": "Alice",
            "age": 30,
            "is_active": true,
            "skills": ["Rust", "C++"]
        }
    "#;

    let mut parser = JsonStreamParser::new(|event| match event {
        JsonEvent::Bool(value) => {
            println!("event with bool {:?}", value)
        }
        JsonEvent::Number(value) => {
            println!("event with float {:?}", value)
        }
        JsonEvent::String(value) => {
            println!("event with string {:?}", value)
        }
        JsonEvent::Key(value) => {
            println!("event with string key {:?}", value)
        }
        _ => println!("event {:?}", event),
    });

    if let Err(err) = parser.parse(json.as_bytes()) {
        eprintln!("Error: {:?}", err);
    }
}
