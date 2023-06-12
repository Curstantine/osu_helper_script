use indicatif::{ProgressBar, ProgressStyle};
use std::io::{Read, Write};

pub fn box_request(request: ureq::Request) -> Result<ureq::Response, Box<ureq::Error>> {
    match request.call() {
        Ok(response) => Ok(response),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn box_and_deserialize<T: for<'a> serde::Deserialize<'a>>(
    request: ureq::Request,
) -> Result<T, Box<ureq::Error>> {
    let response = box_request(request)?;

    let response = match response.into_json::<T>() {
        Ok(response) => response,
        Err(e) => return Err(Box::new(e.into())),
    };

    Ok(response)
}

pub fn download_file_with_progress(
    mut reader: Box<dyn Read + Send + Sync + 'static>,
    size: u64,
) -> Result<Vec<u8>, std::io::Error> {
    let pb = ProgressBar::new(size)
        .with_style(ProgressStyle::with_template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut tracker = ProgressTracker::new(&pb, size);

    // 24MB buffer
    let mut buffer = vec![0; 24 * 1024 * 1024];
    let mut output_buf = Vec::with_capacity(size as usize);

    loop {
        let read_bytes = reader.read(&mut buffer)?;
        if read_bytes == 0 {
            break;
        }

        output_buf.write_all(&buffer[..read_bytes])?;
        tracker.increment(read_bytes as u64);
    }

    tracker.progress_bar.finish();

    Ok(output_buf)
}

pub struct ProgressTracker<'a> {
    progress_bar: &'a ProgressBar,
    downloaded: u64,
}

impl<'a> ProgressTracker<'a> {
    pub fn new(progress_bar: &'a ProgressBar, total: u64) -> Self {
        progress_bar.set_length(total);
        progress_bar.reset_elapsed();
        Self {
            progress_bar,
            downloaded: 0,
        }
    }

    pub fn increment(&mut self, bytes: u64) {
        self.downloaded += bytes;
        self.progress_bar.set_position(self.downloaded);
    }
}
