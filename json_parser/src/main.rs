use std::fs::File;
use crate::json_stream_parser::{JsonEvent, JsonStreamParser};
use crossbeam::channel::{bounded, Sender};
use std::{io, thread};
use std::io::Read;

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

struct Statistic {
    last_price: u64,
    last_qty: u64,
    total_volume: u64,
    total_amount: u64,
    max_bid_price: f64,
    min_ask_price: f64,
    total_trade_count: u64,
}

fn read_bytes(buffer: &mut [u8]) -> usize {
    // Заполняем буфер случайными байтами
    let bytes_read = buffer.len().min(100); // Читаем до 100 байт
    for i in 0..bytes_read {
        buffer[i] = (i % 256) as u8;
    }
    bytes_read
}

fn producer(tx: Sender<Vec<u8>>, file_path: &str, buffer_size: usize) -> io::Result<()> {
    let mut file = File::open(file_path)?;
    let mut buffer = vec![0; buffer_size];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            // File end
            break;
        }

        if tx.send(buffer[..bytes_read].to_vec()).is_err() {
            println!("Consumer dropped. Stopping producer.");
            break;
        }
    }

    Ok(())
}

fn consumer(rx: crossbeam::channel::Receiver<Vec<u8>>) {
    // struct Statistic {
    //     last_price: u64, // last "priceChange": "-16.2038"
    //     last_qty: u64, // last "lastQty": "1000",
    //     total_volume: u64, // "volume": "5",
    //     total_amount: u64, // "amount": "1",
    //     max_bid_price: f64, // "bidPrice":"999.34",
    //     min_ask_price: f64, // "askPrice":"1000.23",
    //     total_trade_count: u64, //  "tradeCount": 5,
    // }

    let mut parser = JsonStreamParser::new(|event| match event {
        JsonEvent::Bool(value) => {
            println!("event with bool {:?}", value)
        }
        JsonEvent::Number(value) => {
            () // println!("event with float {:?}", value)
        }
        JsonEvent::String(value) => {
            () // println!("event with string {:?}", value)
        }
        JsonEvent::Key(value) => {
            () // println!("event with string key {:?}", value)
        }
        _ => (), // println!("event {:?}", event),
    });

    while let Ok(chunk) = rx.recv() {
        if let Err(err) = parser.parse(chunk.as_slice()) {
            eprintln!("Error: {:?}", err);
        }
    }
}

fn main() {
    const QUEUE_SIZE: usize = 1024 * 32;
    const BUFFER_SIZE: usize = 1024 * 32;
    const FILE_PATH: &str = "./ticks.json";

    let (tx, rx) = bounded::<Vec<u8>>(QUEUE_SIZE);

    let producer_thread = thread::spawn(move || {
        if let Err(e) = producer(tx, FILE_PATH, BUFFER_SIZE) {
            eprintln!("Producer encountered an error: {}", e);
        }
    });

    let consumer_thread = thread::spawn(move || {
        consumer(rx);
    });

    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
}
