use std::fmt;

/// Marker for dynamically sized dimensions (no variants).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dynamic {}

impl fmt::Display for Dynamic {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

/// Matrix storage order (column-major or row-major).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Storage {
    /// Column-major layout.
    Column = 0,
    /// Row-major layout.
    Row = 1,
}

impl fmt::Display for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Storage::Column => write!(f, "Column"),
            Storage::Row => write!(f, "Row"),
        }
    }
}

/// Column-major storage constant.
pub const COLUMN_STORAGE: Storage = Storage::Column;
/// Row-major storage constant.
pub const ROW_STORAGE: Storage = Storage::Row;

/// Element initialisation at construction (Armadillo-style). Used by Cube (and optionally Matrix).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    /// Set all elements to zero (default).
    Zeros,
    /// Set all elements to one.
    Ones,
    /// Do not initialise elements (may contain garbage).
    None,
}

impl fmt::Display for Fill {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fill::Zeros => write!(f, "Zeros"),
            Fill::Ones => write!(f, "Ones"),
            Fill::None => write!(f, "None"),
        }
    }
}

/// A sparse matrix entry: value at row `i`, column `j`.
#[derive(Clone, Copy, Debug)]
pub struct Triplet<T> {
    /// The value.
    pub val: T,
    /// Row index.
    pub i: u32,
    /// Column index.
    pub j: u32,
}

impl<T> Triplet<T> {
    /// Creates a triplet (val, i, j).
    pub fn new(val: T, i: u32, j: u32) -> Self {
        Self { val, i, j }
    }
}

impl<T: fmt::Display> fmt::Display for Triplet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.val, self.i, self.j)
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for Triplet<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Triplet", 3)?;
        s.serialize_field("val", &self.val)?;
        s.serialize_field("i", &self.i)?;
        s.serialize_field("j", &self.j)?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Triplet<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Deserialize, Visitor};
        use std::marker::PhantomData;
        struct TripletVisitor<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for TripletVisitor<T> {
            type Value = Triplet<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct Triplet with val, i, j")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Triplet<T>, A::Error> {
                let mut val = None;
                let mut i = None;
                let mut j = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "val" => val = Some(map.next_value()?),
                        "i" => i = Some(map.next_value()?),
                        "j" => j = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let val = val.ok_or_else(|| serde::de::Error::missing_field("val"))?;
                let i = i.ok_or_else(|| serde::de::Error::missing_field("i"))?;
                let j = j.ok_or_else(|| serde::de::Error::missing_field("j"))?;
                Ok(Triplet { val, i, j })
            }
        }
        deserializer.deserialize_struct(
            "Triplet",
            &["val", "i", "j"],
            TripletVisitor::<T>(PhantomData),
        )
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Storage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Storage::Column => serializer.serialize_unit_variant("Storage", 0, "Column"),
            Storage::Row => serializer.serialize_unit_variant("Storage", 1, "Row"),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Storage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Visitor;
        struct StorageVisitor;
        impl<'de> Visitor<'de> for StorageVisitor {
            type Value = Storage;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("enum Storage: Column or Row")
            }
            fn visit_enum<A: serde::de::EnumAccess<'de>>(
                self,
                data: A,
            ) -> Result<Storage, A::Error> {
                let (name, _) = data.variant::<String>()?;
                match name.as_str() {
                    "Column" => Ok(Storage::Column),
                    "Row" => Ok(Storage::Row),
                    _ => Err(serde::de::Error::unknown_variant(&name, &["Column", "Row"])),
                }
            }
        }
        deserializer.deserialize_enum("Storage", &["Column", "Row"], StorageVisitor)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Fill {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Fill::Zeros => serializer.serialize_unit_variant("Fill", 0, "Zeros"),
            Fill::Ones => serializer.serialize_unit_variant("Fill", 1, "Ones"),
            Fill::None => serializer.serialize_unit_variant("Fill", 2, "None"),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Fill {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Visitor;
        struct FillVisitor;
        impl<'de> Visitor<'de> for FillVisitor {
            type Value = Fill;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("enum Fill: Zeros, Ones, None")
            }
            fn visit_enum<A: serde::de::EnumAccess<'de>>(self, data: A) -> Result<Fill, A::Error> {
                let (name, _) = data.variant::<String>()?;
                match name.as_str() {
                    "Zeros" => Ok(Fill::Zeros),
                    "Ones" => Ok(Fill::Ones),
                    "None" => Ok(Fill::None),
                    _ => Err(serde::de::Error::unknown_variant(
                        &name,
                        &["Zeros", "Ones", "None"],
                    )),
                }
            }
        }
        deserializer.deserialize_enum("Fill", &["Zeros", "Ones", "None"], FillVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_enum() {
        assert_eq!(COLUMN_STORAGE, Storage::Column);
        assert_eq!(ROW_STORAGE, Storage::Row);
        assert_ne!(Storage::Column, Storage::Row);
    }

    #[test]
    fn test_storage_display() {
        assert_eq!(format!("{}", Storage::Column), "Column");
        assert_eq!(format!("{}", Storage::Row), "Row");
    }

    #[test]
    fn test_triplet_creation() {
        let t = Triplet::new(2.5_f64, 2, 5);
        assert!((t.val - 2.5).abs() < 1e-10);
        assert_eq!(t.i, 2);
        assert_eq!(t.j, 5);
    }

    #[test]
    fn test_triplet_display() {
        let t = Triplet::new(1.5_f64, 0, 1);
        let s = format!("{}", t);
        assert!(s.contains("1.5"));
        assert!(s.contains('0'));
        assert!(s.contains('1'));
    }

    #[test]
    fn test_triplet_clone() {
        let t1 = Triplet::new(2.0_f64, 1, 2);
        let t2 = t1;
        assert!((t2.val - 2.0).abs() < 1e-10);
        assert_eq!(t2.i, 1);
        assert_eq!(t2.j, 2);
    }
}
