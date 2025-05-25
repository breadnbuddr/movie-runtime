use indexmap::IndexMap;
use reqwest::blocking::get;
use scraper::{CaseSensitivity::CaseSensitive, Html, Selector};

fn main() {
    let url = "https://www.casino-aschaffenburg.de/programm-tickets/#default";
    let response = get(url).expect("Failed to fetch the URL");
    let body = response.text().expect("Failed to read response text");
    let document = Html::parse_document(&body);

    // TODO: programme-table-main-grid-movieitem-data-desktop --> might be relevant for fetching the
    // movie duration

    println!("Fetching events from: {}", url);
    println!("-----------------------------------");
    // println!("{}", body);

    let selector = Selector::parse(
        "div.programme-table-main-date, div.movie-item-caption, div.programme-table-main-movie-item-showtimes"
    ).unwrap();

    let mut structured_data: IndexMap<String, Vec<(String, String)>> = IndexMap::new();
    let mut current_date = String::new();
    let mut elements = document.select(&selector).peekable();

    while let Some(element) = elements.next() {
        if element
            .value()
            .has_class("programme-table-main-date", CaseSensitive)
        {
            current_date = element.text().collect::<String>().trim().to_string();
        } else if element
            .value()
            .has_class("movie-item-caption", CaseSensitive)
        {
            let title = element.text().collect::<String>().trim().to_string();

            if let Some(showtime_element) = elements.peek() {
                if showtime_element
                    .value()
                    .has_class("programme-table-main-movie-item-showtimes", CaseSensitive)
                {
                    let showtime = showtime_element
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string();

                    structured_data
                        .entry(current_date.clone())
                        .or_default()
                        .push((title, showtime));

                    elements.next();
                }
            }
        }
    }

    for (date, movies) in structured_data {
        println!("Date: {}", date);
        for (title, showtime) in movies {
            println!("   Title: {}", title);
            println!("   Showtime: {}", showtime);
        }
        println!("-----------------------------------");
    }
}
