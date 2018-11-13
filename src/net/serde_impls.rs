pub mod aabb {
    use crate::nc::bounding_volume::AABB;
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AABB<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        panic!()
    }

    pub fn serialize<S>(aabb: &AABB<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        panic!()
    }
}
