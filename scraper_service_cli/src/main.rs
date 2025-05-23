use clap::Parser;
use scraper_service_cli::{ai::AIAnalyzer, scraper::Scraper, utils};
use std::env;
use dotenv::dotenv;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to scrape
    #[arg(short = 'u', long)]
    url: String,

    /// Maximum number of pages to scrape
    #[arg(short = 'p', long, default_value_t = 100)]
    max_pages: usize,

    /// Number of concurrent requests
    #[arg(short = 'n', long, default_value_t = 5)]
    concurrent: usize,

    /// Maximum content length per page (in characters)
    #[arg(short = 'l', long, default_value_t = 10000)]
    max_content_length: usize,

    /// Skip AI analysis
    #[arg(short = 's', long)]
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

    let t0 = std::time::Instant::now();
    // 3) Scraper'ı başlat ve sayfaları topla
    let mut scraper = Scraper::new(&args.url, args.max_pages, args.concurrent, args.max_content_length)?;
    let pages = scraper.scrape(&args.url).await?;
    eprintln!("⏱️ Scraping took: {:?}", t0.elapsed());
    if pages.is_empty() {
        eprintln!("❌ Herhangi bir sayfa toplanamadı veya tüm sayfalar korumalıydı.");
        return Ok(());
    }

    // 4) CLI çıktısını JSON olarak hazırlayın
    let mut result = serde_json::Map::new();
    result.insert("pages".to_string(), serde_json::to_value(&pages)?);

    let t1 = std::time::Instant::now();
    // 5) AI analizi yap (eğer atlanmadıysa) ve JSON’a ekle
    if !args.skip_ai {
        let analyzer = AIAnalyzer::new(api_key);
        let analysis = analyzer.analyze_pages(&pages).await?;
        result.insert("analysis".to_string(), serde_json::Value::String(analysis));
    }
    eprintln!("⏱️ AI analysis took: {:?}", t1.elapsed());
    let json_out = serde_json::Value::Object(result);

    // 6) Dosyalara yazmak isterseniz utils’i çağırın (opsiyonel)
    // utils::save_json(&json_out, "result.json")?;
    // if let Some(serde_json::Value::String(ref txt)) = json_out.get("analysis") {
    //     utils::save_text(txt, "analysis.txt")?;
    // }

    // 7) Mutlak JSON’u stdout’a basın
    println!("{}", serde_json::to_string(&json_out)?);
    

    Ok(())
}
