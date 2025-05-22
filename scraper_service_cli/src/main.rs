use clap::Parser;
use scraper_service_cli::{ai::AIAnalyzer, scraper::Scraper, utils};
use std::env;
use dotenv::dotenv;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to scrape
    #[arg(short, long)]
    url: String,

    /// Maximum number of pages to scrape
    #[arg(short, long, default_value_t = 100)]
    max_pages: usize,

    /// Number of concurrent requests
    #[arg(short, long, default_value_t = 5)]
    concurrent: usize,

    /// Maximum content length per page (in characters)
    #[arg(short, long, default_value_t = 10000)]
    max_content_length: usize,

    /// Skip AI analysis
    #[arg(short, long)]
    skip_ai: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) .env'den OpenAI anahtarını oku
    dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY environment variable not set");

    // 2) Komut satırı argümanlarını parse et
    let args = Args::parse();

    // 3) Scraper'ı başlat ve sayfaları topla
    let mut scraper = Scraper::new(&args.url, args.max_pages, args.concurrent, args.max_content_length)?;
    let pages = scraper.scrape(&args.url).await?;

    if pages.is_empty() {
        eprintln!("❌ Herhangi bir sayfa toplanamadı veya tüm sayfalar korumalıydı.");
        return Ok(());
    }

    // 4) JSON çıktısını kaydet
    let output = serde_json::json!({ "pages": pages });
    utils::save_json(&output, "result.json")?;

    // 5) AI analizi yap (eğer atlanmadıysa)
    if !args.skip_ai {
        let analyzer = AIAnalyzer::new(api_key);
        let analysis = analyzer.analyze_pages(&pages).await?;
        utils::save_text(&analysis, "analysis.txt")?;
    }

    Ok(())
}
