use crate::database::models::Faculty;
use scraper::{Html, Selector};

pub async fn scrape_faculties() -> Vec<Faculty> {
    let body = reqwest::get("https://www.rudn.ru/education/schedule")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let document = Html::parse_document(&body);

    // Select 'select' element for faculties
    let faculty_select_element_selector = Selector::parse(r#"select[name="facultet"]"#).unwrap();
    let faculty_select_element = document
        .select(&faculty_select_element_selector)
        .next()
        .unwrap();

    let faculties: Vec<Faculty> = faculty_select_element
        .select(&Selector::parse("option").unwrap())
        .skip(1) // Skip the first element because it is a default option
        .map(|el| {
            let name = el.text().next().unwrap().trim();
            let el = el.value();
            let uuid = el.attr("value").unwrap();
            Faculty {
                uuid: String::from(uuid),
                name: String::from(name),
            }
        })
        .collect();

    faculties
}

pub async fn scrape_groups() {
    todo!()
}

pub async fn scrape_timetables() {
    todo!()
}
