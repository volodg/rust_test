use crate::json_stream_parser::{JsonEvent, JsonStreamParser};
use crossbeam::channel::{bounded, Sender};
use std::fs::File;
use std::io::Read;
use std::{io, thread};

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

#[derive(Debug)]
struct Statistic {
    last_price: f64,
    last_qty: f64,
    total_volume: f64,
    total_amount: f64,
    max_bid_price: f64,
    min_ask_price: f64,
    total_trade_count: u64,
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
        total_volume: 0.0, // TODO use int
        total_amount: 0.0, // TODO use int
        max_bid_price: f64::MIN, // "bidPrice":"999.34",
        min_ask_price: f64::MAX, // "askPrice":"1000.23",
        total_trade_count: 0,    //  "tradeCount": 5,
    };

    let mut parser = JsonStreamParser::new(|event| match event {
        JsonEvent::Number(value) => {
            if let CurrentField::TotalTradeCount = current_field {
                statistic.total_trade_count += value as u64
            }
        }
        JsonEvent::String(value) => {
            match current_field {
                CurrentField::LastPrice => {
                    statistic.last_price = value
                        .parse()
                        .unwrap_or_else(|_| panic!("valid num: {}", value))
                }
                CurrentField::LastQty => {
                    statistic.last_qty = value
                        .parse()
                        .unwrap_or_else(|_| panic!("valid num: {}", value))
                }
                CurrentField::TotalVolume => {
                    statistic.total_volume += value
                        .parse::<f64>()
                        .unwrap_or_else(|_| panic!("valid num: {}", value))
                }
                CurrentField::TotalAmount => {
                    statistic.total_amount += value
                        .parse::<f64>()
                        .unwrap_or_else(|_| panic!("valid num: {}", value))
                }
                CurrentField::MaxBidPrice => {
                    statistic.max_bid_price = statistic.max_bid_price.max(value
                        .parse::<f64>()
                        .unwrap_or_else(|_| panic!("valid num: {}", value)))
                }
                CurrentField::MinAskPrice => {
                    statistic.min_ask_price = statistic.min_ask_price.min(value
                        .parse::<f64>()
                        .unwrap_or_else(|_| panic!("valid num: {}", value)))
                }
                _ => (),
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

    // TODO try to use https://docs.rs/ringbuf/latest/ringbuf/ ?
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
