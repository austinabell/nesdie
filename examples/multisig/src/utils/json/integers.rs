//! Helper classes to serialize and deserialize large integer types into base-10 string
//! representations.
//! NOTE: JSON standard can only work with integer up to 53 bits. So we need helper classes for
//! 64-bit and 128-bit integers.

use crate::alloc::string::ToString;
use borsh::{BorshDeserialize, BorshSerialize};

macro_rules! impl_str_type {
    ($iden: ident, $ty: tt, $place: ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, BorshDeserialize, BorshSerialize)]
        pub struct $iden(pub $ty);

        impl From<$ty> for $iden {
            fn from(v: $ty) -> Self {
                Self(v)
            }
        }

        impl From<$iden> for $ty {
            fn from(v: $iden) -> $ty {
                v.0
            }
        }

        impl miniserde::Serialize for $iden {
            fn begin(&self) -> miniserde::ser::Fragment {
                miniserde::ser::Fragment::Str(self.0.to_string().into())
            }
        }

        miniserde::make_place!($place);
        impl miniserde::de::Visitor for $place<$iden> {
            fn string(&mut self, s: &str) -> miniserde::Result<()> {
                self.out = Some($iden(str::parse::<$ty>(s).map_err(|_| miniserde::Error)?));
                Ok(())
            }
        }

        impl miniserde::Deserialize for $iden {
            fn begin(out: &mut Option<Self>) -> &mut dyn miniserde::de::Visitor {
                $place::new(out)
            }
        }
    };
}

impl_str_type!(U128, u128, P0);
impl_str_type!(U64, u64, P1);
impl_str_type!(I128, i128, P2);
impl_str_type!(I64, i64, P3);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_serde {
        ($str_type: tt, $int_type: tt, $number: expr) => {
            let a: $int_type = $number;
            let str_a: $str_type = a.into();
            let b: $int_type = str_a.into();
            assert_eq!(a, b);

            let str: String = miniserde::json::to_string(&str_a);
            let deser_a: $str_type = miniserde::json::from_str(&str).unwrap();
            assert_eq!(a, deser_a.0);
        };
    }

    #[test]
    fn test_u128() {
        test_serde!(U128, u128, 0);
        test_serde!(U128, u128, 1);
        test_serde!(U128, u128, 123);
        test_serde!(U128, u128, 10u128.pow(18));
        test_serde!(U128, u128, 2u128.pow(100));
        test_serde!(U128, u128, u128::max_value());
    }

    #[test]
    fn test_i128() {
        test_serde!(I128, i128, 0);
        test_serde!(I128, i128, 1);
        test_serde!(I128, i128, -1);
        test_serde!(I128, i128, 123);
        test_serde!(I128, i128, 10i128.pow(18));
        test_serde!(I128, i128, 2i128.pow(100));
        test_serde!(I128, i128, -(2i128.pow(100)));
        test_serde!(I128, i128, i128::max_value());
        test_serde!(I128, i128, i128::min_value());
    }

    #[test]
    fn test_u64() {
        test_serde!(U64, u64, 0);
        test_serde!(U64, u64, 1);
        test_serde!(U64, u64, 123);
        test_serde!(U64, u64, 10u64.pow(18));
        test_serde!(U64, u64, 2u64.pow(60));
        test_serde!(U64, u64, u64::max_value());
    }

    #[test]
    fn test_i64() {
        test_serde!(I64, i64, 0);
        test_serde!(I64, i64, 1);
        test_serde!(I64, i64, -1);
        test_serde!(I64, i64, 123);
        test_serde!(I64, i64, 10i64.pow(18));
        test_serde!(I64, i64, 2i64.pow(60));
        test_serde!(I64, i64, -(2i64.pow(60)));
        test_serde!(I64, i64, i64::max_value());
        test_serde!(I64, i64, i64::min_value());
    }
}
