use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use rand::Rng;

const FILE_NAME: &str = "ticks.json";
const SYMBOL: &str = "BTC-200730-9000-C";
const START_TIME: u64 = 1592317127349;
const ONE_SECOND: u64 = 1000;
const TARGET_SIZE: usize = 1_073_741_824; // 1 GiB

#[derive(Debug)]
struct TradeData {
    price_change: f64,
    price_change_percent: f64,
    last_price: f64,
    last_qty: f64,
    open: f64,
    high: f64,
    low: f64,
    volume: f64,
    amount: f64,
    bid_price: f64,
    ask_price: f64,
    open_time: u64,
    close_time: u64,
    first_trade_id: u64,
    trade_count: u64,
    strike_price: f64,
    exercise_price: f64,
}

impl TradeData {
    fn generate(open_time: u64, close_time: u64) -> Self {
        let mut rng = rand::thread_rng();
        let base_price = rng.gen_range(9000.0..11000.0);

        Self {
            price_change: rng.gen_range(-100.0..100.0),
            price_change_percent: rng.gen_range(-5.0..5.0),
            last_price: base_price,
            last_qty: rng.gen_range(1.0..100.0),
            open: base_price + rng.gen_range(-50.0..50.0),
            high: base_price + rng.gen_range(0.0..100.0),
            low: base_price - rng.gen_range(0.0..100.0),
            volume: rng.gen_range(1.0..1000.0),
            amount: rng.gen_range(1.0..100.0),
            bid_price: base_price - rng.gen_range(0.0..10.0),
            ask_price: base_price + rng.gen_range(0.0..10.0),
            open_time,
            close_time,
            first_trade_id: rng.gen_range(1..1000),
            trade_count: rng.gen_range(1..100),
            strike_price: rng.gen_range(8000.0..12000.0),
            exercise_price: rng.gen_range(3000.0..12000.0),
        }
    }

    fn to_json(&self) -> String {
        format!(
            r#"{{"symbol":"{}","priceChange":"{:.4}","priceChangePercent":"{:.4}","lastPrice":"{:.2}","lastQty":"{:.2}","open":"{:.2}","high":"{:.2}","low":"{:.2}","volume":"{:.2}","amount":"{:.2}","bidPrice":"{:.2}","askPrice":"{:.2}","openTime":{},"closeTime":{},"firstTradeId":{},"tradeCount":{},"strikePrice":"{:.2}","exercisePrice":"{:.4}"}}"#,
            SYMBOL,
            self.price_change,
            self.price_change_percent,
            self.last_price,
            self.last_qty,
            self.open,
            self.high,
            self.low,
            self.volume,
            self.amount,
            self.bid_price,
            self.ask_price,
            self.open_time,
            self.close_time,
            self.first_trade_id,
            self.trade_count,
            self.strike_price,
            self.exercise_price,
        )
    }
}

fn main() -> std::io::Result<()> {
    let file = File::create(FILE_NAME)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "[")?;

    let mut total_size = 0;
    let mut open_time = START_TIME;

    while total_size < TARGET_SIZE {
        let close_time = open_time + ONE_SECOND; // Closes 1 sec after opening
        let trade = TradeData::generate(open_time, close_time);
        let json = trade.to_json();
        total_size += json.len() + 2; // We take into account the length of the record and the comma with a new line
        writeln!(writer, "  {},", json)?;

        open_time = close_time + ONE_SECOND; // Next entry is a second later
    }

    // Remove the last comma and end the JSON array
    writer.seek(std::io::SeekFrom::End(-2))?; // Remove the last comma
    writeln!(writer, "\n]")?;

    writer.flush()?;
    println!("File generated successfully: {}", FILE_NAME);
    Ok(())
}
