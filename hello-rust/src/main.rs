use scraper::{Html, Selector};
    use reqwest::header;

    pub async fn fetch_amazon_price(url: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko)
    Chrome/119.0.0.0 Safari/537.36")
            .build()?;

        let response = client.get(url).send().await?.text().await?;
        let document = Html::parse_document(&response);

        // Селекторы Amazon часто меняются, это пример основного селектора цены
        let selector = Selector::parse(".a-price-whole").unwrap();

        if let Some(element) = document.select(&selector).next() {
            let price_text = element.text().collect::<String>()
                .replace(',', "").replace(' ', "").replace('$', "");

            let price: f64 = price_text.parse()?;
            return Ok(price);
        }

        Err("Price not found".into())
    }

  3. Асинхронный движок (Tokio + Futures)
  Чтобы не ждать каждую из 23 позиций по очереди, используем join_all.

    use futures::future::join_all;

    #[derive(Clone, Debug)]
    pub struct Product {
        pub name: String,
        pub url: String,
        pub current_price: Option<f64>,
    }

    pub async fn update_all_prices(products: &mut Vec<Product>) {
        let mut tasks = Vec::new();

        for product in products.iter() {
            let url = product.url.clone();
            // Создаем задачу на обновление каждой цены
            tasks.push(tokio::spawn(async move {
                fetch_amazon_price(&url).await
            }));
        }

        let results = join_all(tasks).await;

        for (i, res) in results.into_iter().enumerate() {
            if let Ok(Ok(price)) = res {
                products[i].current_price = Some(price);
            }
        }
    }

  4. Визуализация (Ratatui)
  Пример того, как отрисовать таблицу с твоим сетапом.

    use ratatui::{
        widgets::{Block, Borders, Table, Row, Cell},
        layout::{Constraint, Layout},
        Frame,
    };

    pub fn draw_ui(f: &mut Frame, products: &[Product]) {
        let rects = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());

        let header_cells = ["Item", "Price", "Status"]
            .iter()
            .map(|&h| Cell::from(h));

        let rows = products.iter().map(|p| {
            let status = if p.current_price.is_some() { "✅" } else { "❌" };
            let price_str = p.current_price
                .map(|v| format!("${:.2}", v))
                .unwrap_or_else(|| "N/A".to_string());

            Row::new(vec![
                Cell::from(p.name.clone()),
                Cell::from(price_str),
                Cell::from(status),
            ])
        });

        let table = Table::new(rows)
            .header(header_cells)
            .block(Block::default().borders(Borders::ALL).title("Amazon Setup Watcher"))
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(30),
                Constraint::Percentage(20)
            ]);

        f.render_widget(table, rects[0]);
    }