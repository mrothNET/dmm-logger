use std::time::Duration;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub struct MyProgressBar(Option<ProgressBar>);

impl Drop for MyProgressBar {
    fn drop(&mut self) {
        if let Self(Some(bar)) = self {
            bar.finish_and_clear();
        }
    }
}

impl MyProgressBar {
    pub fn new(num_samples: u32) -> MyProgressBar {
        let bar = ProgressBar::new(num_samples.into());

        let spinner = style("{spinner}").green().bold();
        let msg = style("{msg}").bold().magenta();
        let eta = style("({eta})").dim();

        let template = if num_samples == u32::MAX {
            format!("{spinner} [{{elapsed_precise}}] #{{pos}}: {msg}")
        } else {
            format!(
            "{spinner} [{{elapsed_precise}}] {{bar:40.cyan/blue}} #{{pos}}/{{len}}: {msg} {eta}"
        )
        };

        bar.set_style(
            ProgressStyle::with_template(&template)
                .unwrap()
                .progress_chars("=>-"),
        );

        bar.enable_steady_tick(Duration::from_millis(100));

        MyProgressBar(Some(bar))
    }

    pub fn none() -> MyProgressBar {
        MyProgressBar(None)
    }

    pub fn update(&self, reading: f64) {
        if let Self(Some(bar)) = self {
            bar.inc(1);
            bar.set_message(format!("{reading}"));
        }
    }
}
