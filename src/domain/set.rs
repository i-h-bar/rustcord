use uuid::Uuid;

pub struct Set {
    card_id: Uuid,
    name: String,
}

impl Set {
    pub fn new(card_id: Uuid, name: String) -> Self {
        Set { card_id, name }
    }

    pub fn card_id(&self) -> &Uuid {
        &self.card_id
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}