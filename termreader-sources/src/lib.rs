pub mod chapter;
pub mod novel;
pub mod sources;

use anyhow::Result;
use chapter::{Chapter, ChapterPreview};

pub fn get_html<T: reqwest::IntoUrl>(url: T) -> Result<String> {
    let c = reqwest::blocking::ClientBuilder::new().user_agent("Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Mobile Safari/537.36");
    let client = c.build()?;
    let text = client.get(url).send()?.text()?;

    if text.contains("Enable JavaScript and cookies to continue")
        || text.contains("Checking if the site connection is secure")
        || text.contains("Verify below to continue reading")
    {
        panic!("Cloudflare reached, error handling currently unimplemented.");
    } else {
        Ok(text)
    }
}
