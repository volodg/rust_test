use std::cell::RefCell;
use std::rc::Rc;
use criterion::{Criterion, criterion_group, criterion_main};
use json_parser::json_stream_parser::{JsonEvent, JsonStreamParser};

const JSON: &[u8] = r#"
  {
    "symbol": "BTC-200730-9000-C",
    "priceChange": "-16.2038",
    "priceChangePercent": "-0.0162",
    "lastPrice": "1000",
    "lastQty": "1000",
    "open": "1016.2038",
    "high": "1016.2038",
    "low": "0",
    "volume": "5",
    "amount": "1",
    "bidPrice":"999.34",
    "askPrice":"1000.23",
    "openTime": 1592317127349,
    "closeTime": 1592380593516,
    "firstTradeId": 1,
    "tradeCount": 5,
    "strikePrice": "9000",
    "exercisePrice": "3000.3356"
  },
    "#.as_bytes();

fn parse_json<F: for<'a> FnMut(JsonEvent<'a>)>(parser: Rc<RefCell<JsonStreamParser<F>>>) {
    if let Err(err) = parser.borrow_mut().parse(JSON) {
        eprintln!("Error: {:?}", err);
    }
}

// Bench result
// parse_json              time:   [704.66 ns 707.68 ns 710.72 ns]
// with from_utf8_unchecked instead of from_utf8
// parse_json              time:   [470.75 ns 474.51 ns 479.65 ns]

// sudo cargo flamegraph -- --call-graph dwarf
// shows that now most expensive are parse_str and parse::<f64>
fn benchmark_parse_json(c: &mut Criterion) {
    let parser = Rc::new(RefCell::new(JsonStreamParser::new(|_| {})));

    if let Err(err) = parser.borrow_mut().parse(b"[") {
        eprintln!("Error: {:?}", err);
    }

    c.bench_function("parse_json", |b| {
        b.iter(|| parse_json(parser.clone()))
    });
}

criterion_group!(benches, benchmark_parse_json);
criterion_main!(benches);
