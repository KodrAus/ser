#![cfg_attr(not(feature = "std"), no_std)]

/// A serializer for primitive values.
pub trait Serializer {
    /// Serialize a signed integer.
    fn serialize_i64(&mut self, v: i64);
    /// Serialize an unsigned integer.
    fn serialize_u64(&mut self, v: u64);
    /// Serialize a floating point number.
    fn serialize_f64(&mut self, v: f64);
    /// Serialize a boolean.
    fn serialize_bool(&mut self, v: bool);
    /// Serialize a single character.
    fn serialize_char(&mut self, v: char);
    /// Serialize a UTF8 string.
    fn serialize_str(&mut self, v: &str);
    /// Serialize a raw byte buffer.
    fn serialize_bytes(&mut self, v: &[u8]);

    /// Handle an unsupported value.
    fn unsupported(&mut self) {

    }
}

/// A value that can be serialized.
/// 
/// This type is expected to be used as a trait object, like `&dyn Serialize`
/// instead of as a generic, like `T: Serialize`. It is only implemented for
/// a selection of primitive types and cannot be implemented manually.
/// 
/// If the `serde_interop` feature is enabled, this type can be serialized
/// using `serde` in addition to the simple `Serializer` from this crate.
pub trait Serialize: imp::SerializePrivate {
    /// Serialize the value with the given serializer.
    fn serialize(&self, serializer: &mut dyn Serializer);
}

#[cfg(not(feature = "serde_interop"))]
mod imp {
    use super::*;

    #[doc(hidden)]
    pub trait SerializePrivate {}

    macro_rules! ser_primitive {
        ($ty:ty, $($serialize:tt)*) => {
            impl Serialize for $ty {
                $($serialize)*
            }

            impl SerializePrivate for $ty {}
        }
    }

    impl<'a, T: ?Sized> Serialize for &'a T
    where
        T: Serialize,
    {
        fn serialize(&self, serializer: &mut dyn Serializer) {
            (**self).serialize(serializer)
        }
    }

    impl<'a, T: ?Sized> SerializePrivate for &'a T
    where
        T: Serialize,
    {}

    ser_primitive!(u8, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_u64(*self as u64)
    });
    ser_primitive!(u16, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_u64(*self as u64)
    });
    ser_primitive!(u32, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_u64(*self as u64)
    });
    ser_primitive!(u64, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_u64(*self)
    });
    ser_primitive!(i8, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_i64(*self as i64)
    });
    ser_primitive!(i16, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_i64(*self as i64)
    });
    ser_primitive!(i32, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_i64(*self as i64)
    });
    ser_primitive!(i64, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_i64(*self)
    });
    ser_primitive!(f32, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_f64(*self as f64)
    });
    ser_primitive!(f64, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_f64(*self)
    });
    ser_primitive!(char, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_char(*self)
    });
    ser_primitive!(bool, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_bool(*self)
    });
    ser_primitive!(str, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_str(self)
    });
    ser_primitive!([u8], fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_bytes(self)
    });

    #[cfg(feature = "std")]
    ser_primitive!(String, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_str(&*self)
    });
    #[cfg(feature = "std")]
    ser_primitive!(Vec<u8>, fn serialize(&self, serializer: &mut dyn Serializer) {
        serializer.serialize_bytes(&*self)
    });
}

#[cfg(feature = "serde_interop")]
mod imp {
    use super::*;

    use serde::Serializer as SerdeSerializer;

    #[doc(hidden)]
    pub trait SerializePrivate: erased_serde::Serialize {}
 
    impl<T: ?Sized> Serialize for T
    where
        T: serde::Serialize,
    {
        fn serialize(&self, serializer: &mut dyn Serializer) {
            if let Err(Unsupported) = serde::Serialize::serialize(self, SerdeBridge(serializer)) {
                serializer.unsupported();
            }
        }
    }

    impl<T: ?Sized> SerializePrivate for T
    where
        T: serde::Serialize,
    {
    }

    impl<'a> serde::Serialize for dyn Serialize + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            erased_serde::serialize(self, serializer)
        }
    }

    struct SerdeBridge<'a>(&'a mut dyn Serializer);

    #[derive(Debug)]
    struct Unsupported;

    impl serde::ser::Error for Unsupported {
        fn custom<T>(_msg: T) -> Self
        where
            T: std::fmt::Display
        {
            Unsupported
        }
    }

    impl std::fmt::Display for Unsupported {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "unsupported value")
        }
    }

    #[cfg(feature = "std")]
    impl std::error::Error for Unsupported {
        fn cause(&self) -> Option<&dyn std::error::Error> {
            None
        }

        fn description(&self) -> &str {
            "unsupported value"
        }
    }

    impl<'a> serde::Serializer for SerdeBridge<'a> {
        type Ok = ();
        type Error = Unsupported;

        type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.serialize_bool(v))
        }

        fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }

        fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }

        fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }

        fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.serialize_i64(v))
        }

        fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
            self.serialize_u64(v as u64)
        }

        fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
            self.serialize_u64(v as u64)
        }

        fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
            self.serialize_u64(v as u64)
        }

        fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.serialize_u64(v))
        }

        fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
            self.serialize_f64(v as f64)
        }

        fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.serialize_f64(v))
        }

        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.serialize_char(v))
        }

        fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.serialize_str(v))
        }

        fn collect_str<T: std::fmt::Display + ?Sized>(self, _v: &T) -> Result<Self::Ok, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.serialize_bytes(v))
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + serde::Serialize,
        {
            v.serialize(self)
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + serde::Serialize,
        {
            Err(Unsupported)
        }

        fn serialize_newtype_variant<T>(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + serde::Serialize,
        {
            Err(Unsupported)
        }

        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(Unsupported)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn types_implement_serialize() {
        fn is_serialize<T: Serialize + ?Sized>() {}

        macro_rules! assert_is_serialize {
            ($($ty:tt)*) => {
                is_serialize::<$($ty)*>();
                is_serialize::<&$($ty)*>();
            }
        }

        assert_is_serialize!(u8);
        assert_is_serialize!(u16);
        assert_is_serialize!(u32);
        assert_is_serialize!(u64);

        assert_is_serialize!(i8);
        assert_is_serialize!(i16);
        assert_is_serialize!(i32);
        assert_is_serialize!(i64);

        assert_is_serialize!(f32);
        assert_is_serialize!(f64);

        assert_is_serialize!(bool);

        assert_is_serialize!(char);
        assert_is_serialize!(str);
        #[cfg(feature = "std")]
        assert_is_serialize!(String);

        assert_is_serialize!([u8]);
        #[cfg(feature = "std")]
        assert_is_serialize!(Vec<u8>);
    }

    #[derive(PartialEq, Debug)]
    enum Token<'a> {
        I64(i64),
        U64(u64),
        F64(f64),
        Bool(bool),
        Char(char),
        Str(&'a str),
        Bytes(&'a [u8]),
        Unsupported,
    }

    // `&dyn ser::Serialize` should impl `serde::Serialize`
    fn assert_serialize(v: &dyn Serialize, token: Token) {
        struct TestSerializer<'a>(Token<'a>);

        impl<'a> Serializer for TestSerializer<'a> {
            fn serialize_i64(&mut self, v: i64) {
                assert_eq!(self.0, Token::I64(v));
            }
            
            fn serialize_u64(&mut self, v: u64) {
                assert_eq!(self.0, Token::U64(v));
            }

            fn serialize_f64(&mut self, v: f64) {
                assert_eq!(self.0, Token::F64(v));
            }

            fn serialize_bool(&mut self, v: bool) {
                assert_eq!(self.0, Token::Bool(v));
            }

            fn serialize_char(&mut self, v: char) {
                assert_eq!(self.0, Token::Char(v));
            }

            fn serialize_str(&mut self, v: &str) {
                assert_eq!(self.0, Token::Str(v));
            }

            fn serialize_bytes(&mut self, v: &[u8]) {
                assert_eq!(self.0, Token::Bytes(v));
            }

            fn unsupported(&mut self) {
                assert_eq!(self.0, Token::Unsupported);
            }
        }

        v.serialize(&mut TestSerializer(token));
    }

    #[test]
    fn serialize_simple() {
        assert_serialize(&1u8, Token::U64(1u64));
        assert_serialize(&true, Token::Bool(true));
        assert_serialize(&"a string", Token::Str("a string"));
    }

    #[test]
    #[cfg(feature = "serde_interop")]
    fn serialize_unsupported() {
        use serde_json::json;

        let v = json!({
            "id": 123,
            "name": "alice",
        });

        assert_serialize(&v, Token::Unsupported);
    }

    #[cfg(feature = "serde_interop")]
    mod serde_interop {
        use crate::*;
        use serde_test::{Token, assert_ser_tokens};
        use serde_json::json;

        // `&dyn ser::Serialize` should impl `serde::Serialize`
        fn assert_serialize(v: &dyn Serialize, tokens: &[Token]) {
            assert_ser_tokens(&v, tokens);
        }

        #[test]
        fn serialize_simple() {
            assert_serialize(&1u8, &[Token::U8(1u8)]);
            assert_serialize(&true, &[Token::Bool(true)]);
            assert_serialize(&"a string", &[Token::Str("a string")]);
        }

        #[test]
        fn serialize_complex() {
            let v = json!({
                "id": 123,
                "name": "alice",
            });

            assert_serialize(&v, &[
                Token::Map { len: Some(2) },
                Token::Str("id"),
                Token::U64(123),
                Token::Str("name"),
                Token::Str("alice"),
                Token::MapEnd,
            ]);
        }
    }
}
