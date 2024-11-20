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

struct Statistic {
    last_price: u64,
    last_qty: u64,
    total_volume: u64,
    total_amount: u64,
    max_bid_price: f64,
    min_ask_price: f64,
    total_trade_count: f64,
}

/*
[
  {"symbol":"BTC-200730-9000-C","priceChange":"42.7579","priceChangePercent":"0.3388","lastPrice":"10917.96","lastQty":"82.83","open":"10893.02","high":"10943.49","low":"10879.72","volume":"812.39","amount":"56.13","bidPrice":"10909.07","askPrice":"10926.47","openTime":1592317127349,"closeTime":1592317128349,"firstTradeId":319,"tradeCount":91,"strikePrice":"8430.98","exercisePrice":"7419.3571"},
  {"symbol":"BTC-200730-9000-C","priceChange":"40.2731","priceChangePercent":"-1.8285","lastPrice":"9247.74","lastQty":"10.98","open":"9285.10","high":"9255.57","low":"9188.19","volume":"5.72","amount":"25.38","bidPrice":"9243.68","askPrice":"9257.02","openTime":1592317129349,"closeTime":1592317130349,"firstTradeId":477,"tradeCount":4,"strikePrice":"8277.87","exercisePrice":"3540.8722"}
]
 */

fn main() {
    let json = r#"
  [{
    "symbol": "BTC-200730-9000-C",
    "priceChange": "-16.2038",        //24-hour price change
    "priceChangePercent": "-0.0162",  //24-hour percent price change
    "lastPrice": "1000",              //Last trade price
    "lastQty": "1000",                //Last trade amount
    "open": "1016.2038",              //24-hour open price
    "high": "1016.2038",              //24-hour high
    "low": "0",                       //24-hour low
    "volume": "5",                    //Trading volume(contracts)
    "amount": "1",                    //Trade amount(in quote asset)
    "bidPrice":"999.34",              //The best buy price
    "askPrice":"1000.23",             //The best sell price
    "openTime": 1592317127349,        //Time the first trade occurred within the last 24 hours
    "closeTime": 1592380593516,       //Time the last trade occurred within the last 24 hours
    "firstTradeId": 1,                //First trade ID
    "tradeCount": 5,                  //Number of trades
    "strikePrice": "9000",            //Strike price
    "exercisePrice": "3000.3356"      //return estimated settlement price one hour before exercise, return index price at other times
  }]
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
