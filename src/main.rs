use chrono::{Duration, NaiveDate, NaiveTime};
use regex::Regex;
use reqwest::blocking::get;
use scraper::{Html, Selector};
use std::collections::{BTreeMap, HashMap};

fn language_flag(class_attr: &str) -> Option<&'static str> {
    if class_attr.contains("movie-item-showing-lang-OmU") {
        Some("OmU")
    } else if class_attr.contains("movie-item-showing-lang-OV") {
        Some("OV")
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.casino-aschaffenburg.de/programm-tickets/#default";
    let response = get(url).expect("Failed to fetch the URL");
    let body = response.text().expect("Failed to read response text");
    let document = Html::parse_document(&body);

    /* Pass 1: fetch title and runtime */
    let grid_selector = Selector::parse(".programme-table-main-grid-movieitem").unwrap(); // holds all metadata (title, duration)
    let h2_selector = Selector::parse("h2").unwrap(); // movie title is in <h2>
    let span_selector = Selector::parse("span").unwrap(); // every metadata (i.e. duration) is inside a <span>

    let regex_runtime = Regex::new(r"Dauer:\s*(\d+)").unwrap(); // captures the duration of a movie

    let mut movie_runtimes: HashMap<String, i64> = HashMap::new();

    for node in document.select(&grid_selector) {
        let title = node
            .select(&h2_selector)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        if let Some(runtime) = node
            .select(&span_selector)
            .filter_map(|s| {
                regex_runtime
                    .captures(&s.text().collect::<String>())
                    .and_then(|c| c[1].parse::<i64>().ok())
            })
            .next()
        {
            movie_runtimes.insert(title, runtime);
        }
    }

    /* Pass 2: per-date blocks */
    let item_selector = Selector::parse(".programme-table-main-movie-item.movie-item").unwrap(); // holds date, caption, one label per showing time
    let caption_selector = Selector::parse(".movie-item-caption span").unwrap(); // movie title 
    let anchor_selector = Selector::parse("label.movie-item-showtime a").unwrap(); // the class list of <a> holds OmU/OV
    let time_span_selector = Selector::parse("span.movie-itemshowtime-linktext").unwrap(); // showtime of a movie

    type Show = (String, i64, NaiveTime, Option<NaiveTime>, Option<String>);
    let mut showings_by_date: BTreeMap<NaiveDate, Vec<Show>> = BTreeMap::new();

    for item in document.select(&item_selector) {
        // title
        let title = item
            .select(&caption_selector)
            .next()
            .map(|n| n.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        // date
        let day = match item.value().attr("data-date") {
            Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d").ok(),
            None => None,
        };

        // runtime (in minutes) from map
        let rt_minutes = movie_runtimes.get(&title).copied().unwrap_or_default();

        for a in item.select(&anchor_selector) {
            let start_text = a
                .select(&time_span_selector)
                .next()
                .map(|s| s.text().collect::<String>().trim().to_owned())
                .unwrap_or_default();

            let flag = a
                .value()
                .attr("class")
                .and_then(language_flag)
                .map(|s| s.to_string());

            if let (Ok(start), Some(date)) = (NaiveTime::parse_from_str(&start_text, "%H:%M"), day)
            {
                // end time only if runtime is known
                let end = if rt_minutes > 0 {
                    Some(start + Duration::minutes(rt_minutes + 15))
                } else {
                    None
                };
                showings_by_date.entry(date).or_default().push((
                    title.clone(),
                    rt_minutes,
                    start,
                    end,
                    flag,
                ));
            }
        }
    }

    for (day, mut shows) in showings_by_date {
        shows.sort_by_key(|(_, _, start, _, _)| *start);

        println!("{}", day.format("%A, %d %b %Y"));
        for (title, rt, start, end_opt, flag_opt) in shows {
            let start_txt = start.format("%H:%M");

            // end time, if known
            let end_txt = end_opt
                .map(|e| format!("End: {}", e.format("%H:%M")))
                .unwrap_or_else(|| "End:   --".to_string());

            // runtime label
            let dur_txt = if rt > 0 {
                format!("Duration: {} min + 15 min ads", rt)
            } else {
                "Duration:   --".to_string()
            };

            // language suffix
            let flag = flag_opt.map(|f| format!(" [{f}]")).unwrap_or_default();

            println!("{title}");
            println!("    Start: {start_txt}   {end_txt}");
            println!("    {dur_txt}{flag}\n");
        }
    }

    Ok(())
}
