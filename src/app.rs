use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::csvfile;
use crate::instrument;
use crate::scpi;
use crate::status;

pub fn run(
    dmm: &mut scpi::Device,
    mut output: csvfile::CsvFile,
    sample_period: Duration,
    num_samples: u32,
    bar: status::MyProgressBar,
    drop_slow_samples: bool,
) -> Result<()> {
    let term = install_signal_hooks()?;

    let (datetime, started, latency, first_reading) = instrument::read(dmm, 0)?;

    if drop_slow_samples && latency >= sample_period {
        output.write_comment(format!("0: Latency too high! ({})", latency.as_secs_f64()))?;
    } else {
        output.write_reading(0, datetime, 0.0, 0.0, latency.as_secs_f64(), first_reading)?;
    }

    bar.update(first_reading);

    for sequence in 1..num_samples {
        let planed = started + sequence * sample_period;
        let now = Instant::now();

        if drop_slow_samples && now >= planed {
            let delay = (now - planed).as_secs_f64();
            output.write_comment(format!("{sequence}: Too late! {delay}"))?
        } else if sleep_until(planed, &term) {
            let (datetime, moment, latency, reading) = instrument::read(dmm, sequence)?;

            let delay = (moment - planed).as_secs_f64();
            let moment = (moment - started).as_secs_f64();

            if drop_slow_samples && latency >= sample_period {
                output.write_comment(format!(
                    "{sequence}: Latency too high! ({})",
                    latency.as_secs_f64()
                ))?
            } else {
                output.write_reading(
                    sequence,
                    datetime,
                    moment,
                    delay,
                    latency.as_secs_f64(),
                    reading,
                )?;
            }

            bar.update(reading);
        } else {
            break;
        }
    }

    Ok(())
}

fn install_signal_hooks() -> Result<Arc<AtomicBool>> {
    let term = Arc::new(AtomicBool::new(false));
    for signal in signal_hook::consts::TERM_SIGNALS {
        signal_hook::flag::register(*signal, Arc::clone(&term))?;
    }
    Ok(term)
}

fn sleep_until(until: Instant, term: &AtomicBool) -> bool {
    use std::thread::sleep;

    loop {
        if term.load(Ordering::Relaxed) {
            return false;
        }

        let now = Instant::now();
        if now >= until {
            return true;
        }

        let remaining = until - now;
        if remaining.as_millis() >= 150 {
            sleep(Duration::from_millis(100));
        } else {
            sleep(remaining);
        }
    }
}
