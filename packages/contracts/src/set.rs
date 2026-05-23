use time::Date;
use uuid::Uuid;

pub struct Set {
    card_id: Uuid,
    name: String,
    abbreviation: String,
    release_date: Date,
}

impl Set {
    #[must_use]
    pub fn new(card_id: Uuid, name: String, abbreviation: &str, release_date: Date) -> Self {
        Set {
            card_id,
            name,
            abbreviation: abbreviation.to_string(),
            release_date,
        }
    }

    #[must_use]
    pub fn card_id(&self) -> &Uuid {
        &self.card_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn abbreviation(&self) -> &str {
        &self.abbreviation
    }

    #[must_use]
    pub fn release_date(&self) -> &Date {
        &self.release_date
    }
}
