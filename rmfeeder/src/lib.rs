pub mod epub;
pub mod extractor;
pub mod feeds;
pub mod fetcher;
pub mod markdown;
pub mod multipdf;
pub mod pdf;
pub mod state;
pub mod xhtml;
pub mod youtube;

use serde::Deserialize;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
pub struct AppConfig {
    pub state_db_path: Option<String>,
    pub feeds_opml_path: Option<String>,
    pub urls_path: Option<String>,
    pub output_dir: Option<String>,
    pub limit: Option<usize>,
    pub delay: Option<u64>,
    pub summarize: Option<bool>,
    pub pattern: Option<String>,
    pub yt_limit: Option<usize>,
    pub yt_pattern: Option<String>,
    pub yt_delay: Option<u64>,
    pub yt_cookies_browser: Option<String>,
    pub yt_mark_watched_on_success: Option<bool>,
    pub page_size: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageSize {
    Letter,
    Rm1,
    Rm2,
    Rpp,
    RppMove,
    Scribe,
    SupernoteA5x,
    SupernoteA5x2,
    SupernoteA6x,
    SupernoteA6x2,
    BooxGo103,
    BooxNoteAir,
    BooxNoteAir4c,
    BooxNoteAir4cColor,
    BooxNoteMax,
    A6,
    A5,
    A4,
    Ipad11,
    Ipad13,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetSpec {
    pub page_size: PageSize,
    pub flag: &'static str,
    pub aliases: &'static [&'static str],
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: u16,
    pub description: &'static str,
}

impl PageSize {
    pub const VALUE_HINT: &'static str = "letter|rm1|rm2|rmpp|rmpp-move|scribe|supernote-a5x|supernote-a5x2|supernote-a6x|supernote-a6x2|boox-go103|boox-noteair|boox-noteair4c|boox-noteair4c-color|boox-notemax|a6|a5|a4|ipad11|ipad13";
    pub const VALUE_LIST: &'static str = "letter, rm1, rm2, rmpp, rmpp-move, scribe, supernote-a5x, supernote-a5x2, supernote-a6x, supernote-a6x2, boox-go103, boox-noteair, boox-noteair4c, boox-noteair4c-color, boox-notemax, a6, a5, a4, ipad11, ipad13";

    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        Self::all_targets().iter().find_map(|spec| {
            if spec.flag == normalized || spec.aliases.iter().any(|alias| *alias == normalized) {
                Some(spec.page_size)
            } else {
                None
            }
        })
    }

    pub fn as_str(self) -> &'static str {
        self.target_spec().flag
    }

    pub fn css_size_value(self) -> String {
        match self {
            Self::Letter => "letter".to_string(),
            Self::Rm1 => "157.8mm 210.4mm".to_string(),
            Self::Rm2 => "157.8mm 210.4mm".to_string(),
            Self::Rpp => "179.6mm 239.5mm".to_string(),
            Self::RppMove => "179.6mm 239.5mm".to_string(),
            _ => {
                let spec = self.target_spec();
                css_size_from_pixels(spec.width_px, spec.height_px, spec.dpi)
            }
        }
    }

    pub fn page_override_css(self) -> String {
        format!("@page {{ size: {}; }}", self.css_size_value())
    }

    pub fn width_px(self) -> u32 {
        self.target_spec().width_px
    }

    pub fn height_px(self) -> u32 {
        self.target_spec().height_px
    }

    pub fn dpi(self) -> u16 {
        self.target_spec().dpi
    }

    pub fn description(self) -> &'static str {
        self.target_spec().description
    }

    pub fn all_targets() -> &'static [TargetSpec] {
        &TARGET_SPECS
    }

    fn target_spec(self) -> &'static TargetSpec {
        TARGET_SPECS
            .iter()
            .find(|spec| spec.page_size == self)
            .expect("missing target spec for page size")
    }
}

pub fn list_targets_csv() -> String {
    let mut out = String::from("flag,width,height,description\n");
    for spec in PageSize::all_targets() {
        out.push_str(&format!(
            "{},{},{},{}\n",
            spec.flag, spec.width_px, spec.height_px, spec.description
        ));
    }
    out
}

fn css_size_from_pixels(width_px: u32, height_px: u32, dpi: u16) -> String {
    let width_mm = pixels_to_mm(width_px, dpi);
    let height_mm = pixels_to_mm(height_px, dpi);
    format!("{width_mm:.3}mm {height_mm:.3}mm")
}

fn pixels_to_mm(px: u32, dpi: u16) -> f64 {
    (px as f64) * 25.4 / (dpi as f64)
}

const TARGET_SPECS: [TargetSpec; 20] = [
    TargetSpec {
        page_size: PageSize::Letter,
        flag: "letter",
        aliases: &[],
        width_px: 2550,
        height_px: 3300,
        dpi: 300,
        description: "US Letter",
    },
    TargetSpec {
        page_size: PageSize::Rm1,
        flag: "rm1",
        aliases: &["remarkable1", "remarkable-1"],
        width_px: 1404,
        height_px: 1872,
        dpi: 226,
        description: "reMarkable 1",
    },
    TargetSpec {
        page_size: PageSize::Rm2,
        flag: "rm2",
        aliases: &[],
        width_px: 1404,
        height_px: 1872,
        dpi: 226,
        description: "reMarkable 2",
    },
    TargetSpec {
        page_size: PageSize::Rpp,
        flag: "rmpp",
        aliases: &["rpp", "paperpro", "paper-pro", "remarkable-paper-pro"],
        width_px: 1620,
        height_px: 2160,
        dpi: 229,
        description: "reMarkable Paper Pro",
    },
    TargetSpec {
        page_size: PageSize::RppMove,
        flag: "rmpp-move",
        aliases: &[
            "rmppm",
            "rpp-move",
            "paperpro-move",
            "paper-pro-move",
            "remarkable-paper-pro-move",
        ],
        width_px: 1620,
        height_px: 2160,
        dpi: 229,
        description: "reMarkable Paper Pro Move",
    },
    TargetSpec {
        page_size: PageSize::Scribe,
        flag: "scribe",
        aliases: &[],
        width_px: 1860,
        height_px: 2480,
        dpi: 300,
        description: "Kindle Scribe",
    },
    TargetSpec {
        page_size: PageSize::SupernoteA5x,
        flag: "supernote-a5x",
        aliases: &[],
        width_px: 1920,
        height_px: 2560,
        dpi: 226,
        description: "Supernote A5X",
    },
    TargetSpec {
        page_size: PageSize::SupernoteA5x2,
        flag: "supernote-a5x2",
        aliases: &[],
        width_px: 1920,
        height_px: 2560,
        dpi: 226,
        description: "Supernote A5X2",
    },
    TargetSpec {
        page_size: PageSize::SupernoteA6x,
        flag: "supernote-a6x",
        aliases: &[],
        width_px: 1404,
        height_px: 1872,
        dpi: 226,
        description: "Supernote A6X",
    },
    TargetSpec {
        page_size: PageSize::SupernoteA6x2,
        flag: "supernote-a6x2",
        aliases: &[],
        width_px: 1404,
        height_px: 1872,
        dpi: 226,
        description: "Supernote A6X2",
    },
    TargetSpec {
        page_size: PageSize::BooxGo103,
        flag: "boox-go103",
        aliases: &[],
        width_px: 1860,
        height_px: 2480,
        dpi: 300,
        description: "Boox Go 10.3",
    },
    TargetSpec {
        page_size: PageSize::BooxNoteAir,
        flag: "boox-noteair",
        aliases: &[],
        width_px: 1860,
        height_px: 2480,
        dpi: 300,
        description: "Boox Note Air",
    },
    TargetSpec {
        page_size: PageSize::BooxNoteAir4c,
        flag: "boox-noteair4c",
        aliases: &[],
        width_px: 1860,
        height_px: 2480,
        dpi: 300,
        description: "Boox Note Air4 C",
    },
    TargetSpec {
        page_size: PageSize::BooxNoteAir4cColor,
        flag: "boox-noteair4c-color",
        aliases: &[],
        width_px: 930,
        height_px: 1240,
        dpi: 150,
        description: "Boox Note Air4 C Color Layer",
    },
    TargetSpec {
        page_size: PageSize::BooxNoteMax,
        flag: "boox-notemax",
        aliases: &[],
        width_px: 2400,
        height_px: 3200,
        dpi: 300,
        description: "Boox Note Max",
    },
    TargetSpec {
        page_size: PageSize::A6,
        flag: "a6",
        aliases: &[],
        width_px: 1240,
        height_px: 1748,
        dpi: 300,
        description: "ISO A6",
    },
    TargetSpec {
        page_size: PageSize::A5,
        flag: "a5",
        aliases: &[],
        width_px: 1748,
        height_px: 2480,
        dpi: 300,
        description: "ISO A5",
    },
    TargetSpec {
        page_size: PageSize::A4,
        flag: "a4",
        aliases: &[],
        width_px: 2480,
        height_px: 3508,
        dpi: 300,
        description: "ISO A4",
    },
    TargetSpec {
        page_size: PageSize::Ipad11,
        flag: "ipad11",
        aliases: &[],
        width_px: 1668,
        height_px: 2420,
        dpi: 264,
        description: "iPad Pro 11-inch",
    },
    TargetSpec {
        page_size: PageSize::Ipad13,
        flag: "ipad13",
        aliases: &[],
        width_px: 2064,
        height_px: 2752,
        dpi: 264,
        description: "iPad Pro 13-inch",
    },
];

pub fn load_config() -> Result<Option<AppConfig>, Box<dyn std::error::Error>> {
    let path = default_config_path();
    load_config_from_path(path.to_string_lossy().as_ref())
}

pub fn load_config_from_path(path: &str) -> Result<Option<AppConfig>, Box<dyn std::error::Error>> {
    let path = expand_tilde_path(path);
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let cfg: AppConfig = toml::from_str(&contents)?;
            Ok(Some(cfg))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn default_config_path() -> PathBuf {
    if let Some(dir) = default_config_dir() {
        return dir.join("rmfeeder.toml");
    }

    PathBuf::from("rmfeeder.toml")
}

pub fn default_feeds_opml_path() -> PathBuf {
    if let Some(dir) = default_config_dir() {
        return dir.join("feeds.opml");
    }

    PathBuf::from("feeds.opml")
}

pub fn default_config_dir() -> Option<PathBuf> {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME")
        && !xdg_config_home.trim().is_empty()
    {
        return Some(Path::new(&xdg_config_home).join("rmfeeder"));
    }

    if let Ok(home) = std::env::var("HOME") {
        return Some(Path::new(&home).join(".config").join("rmfeeder"));
    }

    None
}

pub fn expand_tilde_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return Path::new(&home).join(rest);
    }
    PathBuf::from(path)
}

/// HTML preview (if you still want it)
pub fn process_url(url: &str) -> String {
    let normalized = match fetcher::normalize_url(url) {
        Ok(u) => u,
        Err(e) => return format!("Invalid URL '{}': {}", url, e),
    };

    let html = match fetcher::fetch_html(&normalized) {
        Ok(body) => body,
        Err(e) => return format!("Fetch error '{}': {}", normalized, e),
    };

    match extractor::extract_article(&html, Some(&normalized)) {
        Some(article) => article.content.to_string(),
        None => format!("Readability failed for {}", normalized),
    }
}

pub fn process_url_to_pdf(url: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    process_url_to_pdf_with_options(url, output_path, false, "summarize", PageSize::Letter)
}

pub fn process_url_to_pdf_with_options(
    url: &str,
    output_path: &str,
    summarize: bool,
    pattern: &str,
    page_size: PageSize,
) -> Result<(), Box<dyn std::error::Error>> {
    let normalized = fetcher::normalize_url(url)?;
    let html = fetcher::fetch_html(&normalized)?;

    if let Some(article) = extractor::extract_article(&html, Some(&normalized)) {
        let body_html = if summarize {
            summarize_html(article.content.as_ref(), &normalized, pattern)?
        } else {
            article.content.to_string()
        };
        pdf::generate_pdf(&article.title, &body_html, output_path, page_size)
    } else {
        Err("Readability extraction failed".into())
    }
}

pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn temp_html_path(prefix: &str) -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let filename = format!("{prefix}_{pid}_{since_epoch}.html");
    std::env::temp_dir().join(filename)
}

pub fn summarize_html(
    content_html: &str,
    source_url: &str,
    pattern: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let summary_html = summarize_content_html(content_html, pattern)?;
    let safe_url = escape_html(source_url);
    let source_html = format!(
        "<p class=\"article-source\">Source: <a href=\"{url}\">{url}</a></p>",
        url = safe_url
    );
    Ok(format!("{}\n{}", source_html, summary_html))
}

pub fn summarize_content_html(
    content_html: &str,
    pattern: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let summary = run_fabric(pattern, content_html)?;
    Ok(markdown::markdown_to_html(&summary))
}

fn run_fabric(pattern: &str, input: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("fabric-ai")
        .arg("-p")
        .arg(pattern)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("fabric failed: {}", stderr.trim()).into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::{PageSize, list_targets_csv};

    #[test]
    fn parses_existing_page_size_values_and_aliases() {
        assert_eq!(PageSize::parse("letter"), Some(PageSize::Letter));
        assert_eq!(PageSize::parse("rm1"), Some(PageSize::Rm1));
        assert_eq!(PageSize::parse("remarkable1"), Some(PageSize::Rm1));
        assert_eq!(PageSize::parse("rmpp"), Some(PageSize::Rpp));
        assert_eq!(PageSize::parse("rmpp-move"), Some(PageSize::RppMove));
        assert_eq!(PageSize::parse("rmppm"), Some(PageSize::RppMove));
        assert_eq!(PageSize::parse("rpp"), Some(PageSize::Rpp));
        assert_eq!(PageSize::parse("rpp-move"), Some(PageSize::RppMove));
        assert_eq!(PageSize::parse("remarkable-paper-pro"), Some(PageSize::Rpp));
        assert_eq!(
            PageSize::parse("remarkable-paper-pro-move"),
            Some(PageSize::RppMove)
        );
    }

    #[test]
    fn preserves_existing_css_size_values_and_page_css() {
        assert_eq!(PageSize::Letter.css_size_value(), "letter");
        assert_eq!(PageSize::Rm1.css_size_value(), "157.8mm 210.4mm");
        assert_eq!(PageSize::Rm2.css_size_value(), "157.8mm 210.4mm");
        assert_eq!(PageSize::Rpp.css_size_value(), "179.6mm 239.5mm");
        assert_eq!(PageSize::RppMove.css_size_value(), "179.6mm 239.5mm");
        assert_eq!(
            PageSize::Letter.page_override_css(),
            "@page { size: letter; }"
        );
        assert_eq!(
            PageSize::Rm1.page_override_css(),
            "@page { size: 157.8mm 210.4mm; }"
        );
        assert_eq!(
            PageSize::RppMove.page_override_css(),
            "@page { size: 179.6mm 239.5mm; }"
        );
    }

    #[test]
    fn parses_new_page_size_values_and_exposes_canonical_names() {
        assert_eq!(PageSize::parse("scribe"), Some(PageSize::Scribe));
        assert_eq!(
            PageSize::parse("supernote-a5x2"),
            Some(PageSize::SupernoteA5x2)
        );
        assert_eq!(
            PageSize::parse("boox-noteair4c-color"),
            Some(PageSize::BooxNoteAir4cColor)
        );
        assert_eq!(PageSize::parse("A6"), Some(PageSize::A6));
        assert_eq!(PageSize::parse("A5"), Some(PageSize::A5));
        assert_eq!(PageSize::parse("A4"), Some(PageSize::A4));
        assert_eq!(PageSize::parse("ipad13"), Some(PageSize::Ipad13));
        assert_eq!(PageSize::Scribe.as_str(), "scribe");
        assert_eq!(PageSize::A4.as_str(), "a4");
        assert_eq!(PageSize::Ipad11.as_str(), "ipad11");
    }

    #[test]
    fn exposes_target_metadata_for_new_page_sizes() {
        assert_eq!(PageSize::Scribe.width_px(), 1860);
        assert_eq!(PageSize::Scribe.height_px(), 2480);
        assert_eq!(PageSize::Scribe.dpi(), 300);
        assert_eq!(PageSize::Scribe.description(), "Kindle Scribe");

        assert_eq!(PageSize::A4.width_px(), 2480);
        assert_eq!(PageSize::A4.height_px(), 3508);
        assert_eq!(PageSize::A4.dpi(), 300);
        assert_eq!(PageSize::A4.description(), "ISO A4");
    }

    #[test]
    fn computes_css_sizes_for_new_page_sizes_from_pixels_and_dpi() {
        assert_eq!(PageSize::Scribe.css_size_value(), "157.480mm 209.973mm");
        assert_eq!(
            PageSize::SupernoteA5x.css_size_value(),
            "215.788mm 287.717mm"
        );
        assert_eq!(
            PageSize::BooxNoteAir4cColor.css_size_value(),
            "157.480mm 209.973mm"
        );
        assert_eq!(PageSize::A4.css_size_value(), "209.973mm 297.011mm");
        assert_eq!(PageSize::Ipad11.css_size_value(), "160.482mm 232.833mm");
    }

    #[test]
    fn exposes_stable_target_listing_output() {
        let listing = list_targets_csv();
        assert!(listing.starts_with("flag,width,height,description\n"));
        assert!(listing.contains("letter,2550,3300,US Letter\n"));
        assert!(listing.contains("rmpp-move,1620,2160,reMarkable Paper Pro Move\n"));
        assert!(listing.contains("boox-noteair4c-color,930,1240,Boox Note Air4 C Color Layer\n"));
        assert!(listing.contains("a4,2480,3508,ISO A4\n"));
        assert!(listing.contains("ipad13,2064,2752,iPad Pro 13-inch\n"));
        assert_eq!(PageSize::all_targets().len(), 20);
    }
}
