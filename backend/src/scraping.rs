use std::collections::HashMap;

use crate::database::models::*;
use scraper::{Html, Selector};

pub async fn scrape_faculties() -> Vec<Faculty> {
    log::info!("Scraping faculties");
    let response = reqwest::get("https://www.rudn.ru/education/schedule")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let document = Html::parse_document(&response);

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

pub async fn scrape_timetables() {
    todo!()
}
