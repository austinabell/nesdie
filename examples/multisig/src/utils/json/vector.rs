extern crate alloc;

use alloc::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};
use miniserde::de::Visitor;
use miniserde::make_place;
use miniserde::ser::Fragment;

/// Helper class to serialize/deserialize `Vec<u8>` to base64 string.
#[derive(Debug, Clone, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Base64VecU8(pub Vec<u8>);

impl From<Vec<u8>> for Base64VecU8 {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<Base64VecU8> for Vec<u8> {
    fn from(v: Base64VecU8) -> Vec<u8> {
        v.0
    }
}

impl miniserde::Serialize for Base64VecU8 {
    fn begin(&self) -> Fragment {
        Fragment::Str(base64::encode(&self.0).into())
    }
}

make_place!(Place);
impl Visitor for Place<Base64VecU8> {
    fn string(&mut self, s: &str) -> miniserde::Result<()> {
        self.out = Some(Base64VecU8(
            base64::decode(s).map_err(|_| miniserde::Error)?,
        ));
        Ok(())
    }
}

impl miniserde::Deserialize for Base64VecU8 {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        Place::new(out)
    }
}

// mod base64_bytes {
//     use super::*;
//     use serde::{de, ser};

//     pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: ser::Serializer,
//     {
//         serializer.serialize_str(&base64::encode(&bytes))
//     }

//     pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
//     where
//         D: de::Deserializer<'de>,
//     {
//         let s: alloc::string::String = Deserialize::deserialize(deserializer)?;
//         base64::decode(s.as_str()).map_err(de::Error::custom)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use miniserde::json;

    macro_rules! test_serde {
        ($v: expr) => {
            let a: Vec<u8> = $v;
            let wrapped_a: Base64VecU8 = a.clone().into();
            let b: Vec<u8> = wrapped_a.clone().into();
            assert_eq!(a, b);

            let s: String = json::to_string(&wrapped_a);
            let deser_a: Base64VecU8 = json::from_str(&s).unwrap();
            assert_eq!(a, deser_a.0);
        };
    }

    #[test]
    fn test_empty() {
        test_serde!(vec![]);
    }

    #[test]
    fn test_basic() {
        test_serde!(vec![0]);
        test_serde!(vec![1]);
        test_serde!(vec![1, 2, 3]);
        test_serde!(b"abc".to_vec());
        test_serde!(vec![3, 255, 255, 13, 0, 23]);
    }

    #[test]
    fn test_long() {
        test_serde!(vec![123; 16000]);
    }

    // #[test]
    // fn test_manual() {
    //     let a = vec![100, 121, 31, 20, 0, 23, 32];
    //     let a_str = json::to_string::<_, 20>(&Base64VecU8(a.clone())).unwrap();
    //     assert_eq!(a_str, String::from("\"ZHkfFAAXIA==\""));
    //     let a_deser: Base64VecU8 = json::from_str(&a_str).unwrap().0;
    //     assert_eq!(a_deser.0, a);
    // }
}
