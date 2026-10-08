use std::io;

use indicatif::{ProgressBar, ProgressStyle};

pub fn download(url: &str, name: &str) -> Option<Vec<u8>> {
    let mut buffer = Vec::new();

    let mut response = match ureq::get(url).call() {
        Ok(resp) => resp,
        Err(_) => return None,
    };
    let total_size = match response.headers().get("Content-Length") {
        Some(len) => match len.to_str().unwrap_or("0").parse::<u64>() {
            Ok(len) => len,
            Err(_) => 0,
        },
        None => 0,
    };

    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{wide_msg} {bytes} {bytes_per_sec} {elapsed_precise} {bar} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    progress_bar.set_message(format!("    {}", name));

    let reader = response.body_mut().as_reader();
    if let Err(_) = io::copy(&mut progress_bar.wrap_read(reader), &mut buffer) {
        return None;
    }

    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{wide_msg} {bytes} {elapsed_precise} {bar} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    progress_bar.finish();

    Some(buffer)
}
