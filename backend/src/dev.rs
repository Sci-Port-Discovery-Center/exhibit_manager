use crate::api::exhibit_handlers::NewExhibit;
use rand::prelude::IndexedRandom;
use rocket::serde;

const EXHIBIT_DATA: &'static str = include_str!("../exhibit_dev_data/exhibits.json");

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DummyExhibitData {
    exhibit_name: String,
    exhibit_description: String,
    building_location: String,
    current_status: String,
    cluster: String,
    _notes: String,
}

pub fn get_random_dummy_exhibit() -> NewExhibit {
    let exhibits: Vec<DummyExhibitData> = serde::json::serde_json::from_str(EXHIBIT_DATA).unwrap();
    let dummy_exhibit_data = exhibits.choose(&mut rand::rng()).unwrap();

    let exhibit = NewExhibit {
        name: dummy_exhibit_data.exhibit_name.clone(),
        cluster: dummy_exhibit_data.cluster.clone(),
        location: dummy_exhibit_data.building_location.clone(),
        description: dummy_exhibit_data.exhibit_description.clone(),
        status: dummy_exhibit_data.current_status.clone(),
        image_url: Some(format!(
            "https://picsum.photos/seed/{}/200/300",
            dummy_exhibit_data.exhibit_name
        )),
        sponsor: None,
        part_ids: vec![],
        notes: vec![],
    };

    exhibit
}
