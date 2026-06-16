use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait Storage {
    async fn get_set_volumes(&self, sets: Vec<Set>) -> Vec<(Set, u32)>;
}


pub struct Set {
    pub id: Uuid,
    pub name: String,
    pub normalised_name: String,
    pub abbreviation: String,
}

pub struct Card {
    pub id: Uuid,
    pub name: String,
}