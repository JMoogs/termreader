pub mod chapter;
pub mod novel;
pub mod sources;

use anyhow::Result;
use chapter::{Chapter, ChapterPreview};
use std::fmt::Debug;

pub fn get_html<T: reqwest::IntoUrl + Debug>(url: T) -> Result<String> {
    let c = reqwest::blocking::ClientBuilder::new()
        .use_rustls_tls()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0");
    let client = c.build()?;
    let text = client.get(url).send()?.text()?;

    if text.contains("Enable JavaScript and cookies to continue")
        || text.contains("Checking if the site connection is secure")
        || text.contains("Verify below to continue reading")
    {
        eprintln!("ERROR:\n\n\n\n{}\n\n\n\n", text);
        panic!("Cloudflare reached, error handling currently unimplemented.");
    } else {
        Ok(text)
    }
}
