use std::collections::HashMap;

use crate::database::models::*;
use chrono::{Datelike, NaiveTime};
use scraper::{Html, Selector};

pub async fn scrape_faculties() -> Option<Vec<Faculty>> {
    log::info!("Scraping faculties");
    let response = reqwest::get("https://www.rudn.ru/education/schedule")
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let document = Html::parse_document(&response);

    // Select 'select' element for faculties
    let faculty_select_element_selector = Selector::parse(r#"select[name="facultet"]"#).ok()?;
    let faculty_select_element = document.select(&faculty_select_element_selector).next()?;

    let faculties: Vec<Faculty> = faculty_select_element
        .select(&Selector::parse("option").ok()?)
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

    Some(faculties)
}

pub async fn scrape_groups(faculties_uuid: &Vec<Uuid>) -> HashMap<Uuid, Vec<Group>> {
    log::info!("Scraping groups for {faculties_uuid:?}");
    let mut output = HashMap::new();
    for uuid in faculties_uuid {
        let mut payload = HashMap::new();
        payload.insert("facultet", uuid.clone());
        payload.insert("level", String::from(""));
        payload.insert("action", String::from("filterData"));
        let groups = match reqwest::Client::new()
            .post("https://www.rudn.ru/api/v1/education/schedule")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                let parsed = json::parse(&resp.text().await.expect("There is no body"))
                    .expect("Json error in response");
                match &parsed["data"]["elements"]["group"]["list"] {
                    json::JsonValue::Array(vec) => {
                        let mut groups = vec![];
                        for el in vec {
                            let group = Group {
                                uuid: el["value"].as_str().unwrap().to_string(),
                                name: el["name"].as_str().unwrap().to_string(),
                                faculty: uuid.clone(),
                            };
                            groups.push(group);
                        }

                        groups
                    }
                    t => {
                        log::error!("Unexpected group list format: {t:?}");
                        vec![]
                    }
                }
            }

            Err(e) => {
                log::error!("{e:?}");
                vec![]
            }
        };

        output.insert(uuid.clone(), groups);
    }

    output
}

pub async fn scrape_timetable(group_uuid: Uuid) -> anyhow::Result<HashMap<Day, Vec<Event>>> {
    let response = reqwest::get(format!(
        "https://www.rudn.ru/api/v1/education/schedule?group={group_uuid}"
    ))
    .await?
    .text()
    .await?;

    let document = Html::parse_document(&response);

    let current_week_table = {
        let curr_week_number = chrono::Local::now().iso_week().week();
        let current_week_tabpanel_selector =
            Selector::parse(&format!("#tab__level-{curr_week_number}")).unwrap();
        let current_week_tabpanel = document
            .select(&current_week_tabpanel_selector)
            .next()
            .ok_or(anyhow::anyhow!("No week tabpanel"))?;
        current_week_tabpanel
            .select(&Selector::parse("table").unwrap())
            .next()
            .ok_or(anyhow::anyhow!("No week table"))?
    };

    let mut day = Day::Monday;
    let mut skip_to_next_day = false;

    let classes = current_week_table
        .select(&Selector::parse("tr").unwrap())
        .fold(HashMap::new(), |mut map: HashMap<Day, Vec<Event>>, el| {
            match el.select(&Selector::parse("th").unwrap()).next() {
                Some(el) => match Day::from_russian(&el.inner_html()) {
                    Ok(d) => {
                        day = d;
                        skip_to_next_day = false;
                    }
                    Err(_) => {
                        skip_to_next_day = true;
                    }
                },
                None => {
                    if !skip_to_next_day {
                        let time_el = el
                            .select(&Selector::parse(r#".edss__table-time"#).unwrap())
                            .next();
                        let name_el = el
                            .select(&Selector::parse(r#".edss__table-subj"#).unwrap())
                            .next();

                        if let (Some(time_el), Some(name_el)) = (time_el, name_el) {
                            let time = time_el
                                .inner_html()
                                .split(" - ")
                                .map(|el| NaiveTime::parse_from_str(el, "%H:%M").unwrap())
                                .collect::<Vec<_>>();

                            let event = Event {
                                name: name_el.inner_html(),
                                day,
                                start_time: time[0],
                                end_time: time[1],
                                student_group: group_uuid.clone(),
                            };

                            map.entry(day)
                                .and_modify(|events| events.push(event.clone()))
                                .or_insert_with(|| vec![event]);
                        }
                    }
                }
            }
            map
        });

    Ok(classes)
}
