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

#[derive(Debug)]
struct Statistic {
    last_price: f64,
    last_qty: f64,
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
    enum CurrentField {
        Unknown,
        LastPrice,
        LastQty,
        TotalVolume,
        TotalAmount,
        MaxBidPrice,
        MinAskPrice,
        TotalTradeCount,
    }


    let mut current_field = CurrentField::Unknown;
    let mut statistic = Statistic {
        last_price: 0.0,
        last_qty: 0.0,
        total_volume: 0, // "volume": "5",
        total_amount: 0, // "amount": "1",
        max_bid_price: f64::MIN, // "bidPrice":"999.34",
        min_ask_price: f64::MAX, // "askPrice":"1000.23",
        total_trade_count: 0, //  "tradeCount": 5,
    };

    let mut parser = JsonStreamParser::new(|event| match event {
        JsonEvent::Number(value) => {
            match current_field {
                CurrentField::LastPrice => (),
                CurrentField::LastQty => (),
                CurrentField::TotalVolume => (),
                CurrentField::TotalAmount => (),
                CurrentField::MaxBidPrice => (),
                CurrentField::MinAskPrice => (),
                CurrentField::TotalTradeCount => (),
                CurrentField::Unknown => (),
            };
        }
        JsonEvent::String(value) => {
            match current_field {
                CurrentField::LastPrice => {
                    statistic.last_price = value.parse().expect(format!("valid num: {}", value).as_str())
                },
                CurrentField::LastQty => {
                    statistic.last_qty = value.parse().expect(format!("valid num: {}", value).as_str())
                },
                CurrentField::TotalVolume => (),
                CurrentField::TotalAmount => (),
                CurrentField::MaxBidPrice => (),
                CurrentField::MinAskPrice => (),
                CurrentField::TotalTradeCount => (),
                CurrentField::Unknown => (),
            };
        }
        JsonEvent::Key(value) => {
            current_field = match value {
                "lastPrice" => CurrentField::LastPrice,
                "lastQty" => CurrentField::LastQty,
                "volume" => CurrentField::TotalVolume,
                "amount" => CurrentField::TotalAmount,
                "bidPrice" => CurrentField::MaxBidPrice,
                "askPrice" => CurrentField::MinAskPrice,
                "total_trade_count" => CurrentField::TotalTradeCount,
                _ => CurrentField::Unknown,
            };
        }
        _ => (),
    });

    while let Ok(chunk) = rx.recv() {
        if let Err(err) = parser.parse(chunk.as_slice()) {
            eprintln!("Error: {:?}", err);
        }
    }

    println!("statistic: {:?}", statistic)
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
