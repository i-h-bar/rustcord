use uuid::Uuid;

pub fn increment_uuid(id: Uuid) -> Uuid {
    Uuid::from_u128(id.as_u128().wrapping_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_basic() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        assert_eq!(
            increment_uuid(id),
            Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
        );
    }

    #[test]
    fn test_increment_realistic_uuid() {
        let id = Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91e").unwrap();
        assert_eq!(
            increment_uuid(id),
            Uuid::parse_str("0000419b-0bba-4488-8f7a-6194544ce91f").unwrap()
        );
    }

    #[test]
    fn test_increment_wraps_on_overflow() {
        let id = Uuid::from_u128(u128::MAX);
        assert_eq!(increment_uuid(id), Uuid::from_u128(0));
    }
}
