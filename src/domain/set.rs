use uuid::Uuid;

pub struct Set {
    card_id: Uuid,
    name: String,
}

impl Set {
    #[must_use]
    pub fn new(card_id: Uuid, name: String) -> Self {
        Set { card_id, name }
    }

    #[must_use]
    pub fn card_id(&self) -> &Uuid {
        &self.card_id
    }

    #[must_use]
    pub fn name(&self) -> &String {
        &self.name
    }
}
