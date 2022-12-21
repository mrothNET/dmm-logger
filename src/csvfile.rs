use chrono::prelude::*;

pub fn write_header(ident: &str) {
    println!("# Identification: {ident}");
    println!("sequence,date,time,moment,delay,latency,reading");
}

pub fn write_line(
    sequence: u32,
    datetime: DateTime<Local>,
    moment: f64,
    delay: f64,
    latency: f64,
    reading: f64,
) {
    let date = datetime.format("%Y-%m-%d");
    let time = datetime.format("%H:%M:%S.%3f");
    println!("{sequence},{date},{time},{moment},{delay},{latency},{reading}");
}
