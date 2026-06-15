use uuid::Uuid;


#[derive(Default)]
pub struct Set {
    pub id: Uuid,
    pub name: String,
    pub abbreviation: String,
    pub normalised_name: String,
    pub card_count: Option<u32>,
}