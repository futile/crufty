pub mod aabb {
    use crate::nc::bounding_volume::AABB;
    use serde::{Deserializer, Serializer};
    use serde::ser::SerializeStruct;

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
        let mut state = serializer.serialize_struct("AABB", 2)?;
        state.serialize_field("mins", aabb.mins());
        state.serialize_field("maxs", aabb.maxs());
        state.end()
    }
}
