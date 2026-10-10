use std::io;

use indicatif::{ProgressBar, ProgressStyle};
use ureq::{Body, http::Response};

pub fn download(url: &str, name: &str) -> Option<Vec<u8>> {
    let response = match get_response(url) {
        Some(resp) => resp,
        None => return None,
    };

    download_from_previous(response, name)
}

pub fn get_response(url: &str) -> Option<Response<Body>> {
    let response = match ureq::get(url).call() {
        Ok(resp) => resp,
        Err(_) => return None,
    };

    return Some(response);
}

pub fn get_size(response: &Response<Body>) -> Option<u64> {
    match response.headers().get("Content-Length") {
        Some(len) => match len.to_str().unwrap_or("0").parse::<u64>() {
            Ok(len) => Some(len),
            Err(_) => None,
        },
        None => None,
    }
}

pub fn download_from_previous(mut response: Response<Body>, name: &str) -> Option<Vec<u8>> {
    let mut buffer = Vec::new();

    let progress_bar = ProgressBar::new(get_size(&response).unwrap_or(0));
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
