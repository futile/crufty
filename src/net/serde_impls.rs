pub mod aabb {
    use crate::na::Point2;
    use crate::nc::bounding_volume::AABB;
    use serde::{Deserializer, Serializer, Serialize, Deserialize};

    #[derive(Serialize, Deserialize)]
    struct Bounds {
        mins: Point2<f32>,
        maxs: Point2<f32>,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AABB<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Bounds {mins, maxs} = Bounds::deserialize(deserializer)?;

        Ok(AABB::new(mins, maxs))
    }

    pub fn serialize<S>(aabb: &AABB<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bounds = Bounds {
            mins: *aabb.mins(),
            maxs: *aabb.maxs(),
        };

        bounds.serialize(serializer)
    }
}

pub mod cuboid {
    use crate::na::Vector2;
    use crate::nc::shape::Cuboid;
    use serde::{Deserializer, Serializer, Serialize, Deserialize};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Cuboid<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vector2::deserialize(deserializer).map(Cuboid::new)
    }

    pub fn serialize<S>(cuboid: &Cuboid<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        cuboid.half_extents().serialize(serializer)
    }
}

pub mod smallvec {
    use smallvec::SmallVec;
    use serde::{Deserializer, Serializer, Serialize, Deserialize};

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<SmallVec<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + smallvec::Array,
    {
        // Vector2::deserialize(deserializer).map(Cuboid::new)
        panic!()
    }

    pub fn serialize<S, T>(sv: &SmallVec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize + smallvec::Array,
    {
        panic!()
        // cuboid.half_extents().serialize(serializer)
    }
}
