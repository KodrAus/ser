#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate core as std;

#[cfg(feature = "std")]
extern crate std;

use self::std::fmt;

/// A serializer for primitive values.
pub trait Visitor {
    /// Visit a signed integer.
    fn visit_i64(&mut self, v: i64) {
        self.visit_args(&format_args!("{:?}", v));
    }

    /// Visit an unsigned integer.
    fn visit_u64(&mut self, v: u64) {
        self.visit_args(&format_args!("{:?}", v));
    }

    /// Visit a floating point number.
    fn visit_f64(&mut self, v: f64) {
        self.visit_args(&format_args!("{:?}", v));
    }

    /// Visit a boolean.
    fn visit_bool(&mut self, v: bool) {
        self.visit_args(&format_args!("{:?}", v));
    }

    /// Visit a single character.
    fn visit_char(&mut self, v: char) {
        let mut b = [0; 4];
        self.visit_str(&*v.encode_utf8(&mut b));
    }

    /// Visit a UTF8 string.
    fn visit_str(&mut self, v: &str) {
        self.visit_args(&format_args!("{:?}", v));
    }

    /// Visit a raw byte buffer.
    fn visit_bytes(&mut self, v: &[u8]) {
        self.visit_args(&format_args!("{:?}", v));
    }

    /// Visit standard arguments.
    fn visit_args(&mut self, args: &fmt::Arguments);
}

/// A value that can be serialized.
/// 
/// This type is expected to be used as a trait object, like `&dyn Visit`
/// instead of as a generic, like `T: Visit`. It is only implemented for
/// a selection of primitive types and cannot be implemented manually.
/// 
/// If the `serde_interop` feature is enabled, this type can be serialized
/// using `serde` in addition to the simple `Visitor` from this crate.
pub trait Visit: imp::VisitPrivate {
    /// Visit the value with the given serializer.
    fn visit(&self, visitor: &mut dyn Visitor);
}

trait EnsureVisit: Visit {}

macro_rules! ensure_impl_visit {
    ($($ty:ty { $($serialize:tt)* })*) => {
        $(
            #[cfg(feature = "serde_interop")]
            impl EnsureVisit for $ty {}

            #[cfg(not(feature = "serde_interop"))]
            impl Visit for $ty {
                $($serialize)*
            }

            #[cfg(not(feature = "serde_interop"))]
            impl imp::VisitPrivate for $ty {}
        )*
    }
}

ensure_impl_visit! {
    u8 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_u64(*self as u64)
        }
    }
    u16 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_u64(*self as u64)
        }
    }
    u32 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_u64(*self as u64)
        }
    }
    u64 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_u64(*self)
        }
    }

    i8 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_i64(*self as i64)
        }
    }
    i16 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_i64(*self as i64)
        }
    }
    i32 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_i64(*self as i64)
        }
    }
    i64 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_i64(*self)
        }
    }

    f32 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_f64(*self as f64)
        }
    }
    f64 {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_f64(*self)
        }
    }

    char {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_char(*self)
        }
    }
    bool {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_bool(*self)
        }
    }
    str {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_str(self)
        }
    }
    [u8] {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_bytes(self)
        }
    }
}

#[cfg(feature = "std")]
ensure_impl_visit! {
    String {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_str(&*self)
        }
    }
    Vec<u8> {
        fn visit(&self, visitor: &mut dyn Visitor) {
            visitor.visit_bytes(&*self)
        }
    }
}

#[cfg(not(feature = "serde_interop"))]
mod imp {
    use super::*;

    #[doc(hidden)]
    pub trait VisitPrivate: fmt::Debug {}

    impl<'a, T: ?Sized> Visit for &'a T
    where
        T: Visit,
    {
        fn visit(&self, visitor: &mut dyn Visitor) {
            (**self).visit(visitor)
        }
    }

    impl<'a, T: ?Sized> VisitPrivate for &'a T
    where
        T: Visit,
    {
    }
}

#[cfg(feature = "serde_interop")]
mod imp {
    use super::*;

    use serde::{Serializer, Serialize};

    #[doc(hidden)]
    pub trait VisitPrivate: erased_serde::Serialize + fmt::Debug {}
 
    impl<T: ?Sized> Visit for T
    where
        T: Serialize + fmt::Debug,
    {
        fn visit(&self, visitor: &mut dyn Visitor) {
            if let Err(Unsupported) = Serialize::serialize(self, SerdeBridge(visitor)) {
                visitor.visit_args(&format_args!("{:?}", self));
            }
        }
    }

    impl<T: ?Sized> VisitPrivate for T
    where
        T: Serialize + fmt::Debug,
    {
    }

    impl<'a> Serialize for dyn Visit + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            erased_serde::serialize(self, serializer)
        }
    }

    struct SerdeBridge<'a>(&'a mut dyn Visitor);

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

    impl<'a> Serializer for SerdeBridge<'a> {
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
            Ok(self.0.visit_bool(v))
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
            Ok(self.0.visit_i64(v))
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
            Ok(self.0.visit_u64(v))
        }

        fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
            self.serialize_f64(v as f64)
        }

        fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.visit_f64(v))
        }

        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.visit_char(v))
        }

        fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.visit_str(v))
        }

        fn collect_str<T: std::fmt::Display + ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.visit_args(&format_args!("{}", v)))
        }

        fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
            Ok(self.0.visit_bytes(v))
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Err(Unsupported)
        }

        fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
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
            T: ?Sized + Serialize,
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
            T: ?Sized + Serialize,
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

    #[derive(PartialEq, Debug)]
    enum Token<'a> {
        I64(i64),
        U64(u64),
        F64(f64),
        Bool(bool),
        Char(char),
        Str(&'a str),
        Bytes(&'a [u8]),
        Args(&'a str),
    }

    // `&dyn ser::Serialize` should impl `Serialize`
    fn assert_visit(v: &dyn Visit, token: Token) {
        struct TestVisitor<'a>(Token<'a>);

        impl<'a> Visitor for TestVisitor<'a> {
            fn visit_i64(&mut self, v: i64) {
                assert_eq!(self.0, Token::I64(v));
            }
            
            fn visit_u64(&mut self, v: u64) {
                assert_eq!(self.0, Token::U64(v));
            }

            fn visit_f64(&mut self, v: f64) {
                assert_eq!(self.0, Token::F64(v));
            }

            fn visit_bool(&mut self, v: bool) {
                assert_eq!(self.0, Token::Bool(v));
            }

            fn visit_char(&mut self, v: char) {
                assert_eq!(self.0, Token::Char(v));
            }

            fn visit_str(&mut self, v: &str) {
                assert_eq!(self.0, Token::Str(v));
            }

            fn visit_bytes(&mut self, v: &[u8]) {
                assert_eq!(self.0, Token::Bytes(v));
            }

            fn visit_args(&mut self, v: &fmt::Arguments) {
                use self::std::{str, ptr};
                use self::fmt::Write;

                const LEN: usize = 128;

                struct VisitArgs {
                    buf: [u8; LEN],
                    cursor: usize,
                }

                impl VisitArgs {
                    fn new() -> Self {
                        VisitArgs {
                            buf: [0; LEN],
                            cursor: 0,
                        }
                    }

                    fn to_str(&self) -> Option<&str> {
                        str::from_utf8(&self.buf[0..self.cursor]).ok()
                    }
                }

                impl Write for VisitArgs {
                    fn write_str(&mut self, s: &str) -> fmt::Result {
                        let src = s.as_bytes();
                        let next_cursor = self.cursor + src.len();

                        if next_cursor > LEN {
                            return Err(fmt::Error);
                        }

                        unsafe {
                            let src_ptr = src.as_ptr();
                            let dst_ptr = self.buf.as_mut_ptr().offset(self.cursor as isize);

                            ptr::copy_nonoverlapping(src_ptr, dst_ptr, src.len());
                        }

                        self.cursor = next_cursor;

                        Ok(())
                    }
                }

                let mut w = VisitArgs::new();
                write!(&mut w, "{}", v).unwrap();

                assert_eq!(self.0, Token::Args(w.to_str().unwrap()));
            }
        }

        v.visit(&mut TestVisitor(token));
    }

    #[test]
    fn visit_simple() {
        assert_visit(&1u8, Token::U64(1u64));
        assert_visit(&true, Token::Bool(true));
        assert_visit(&"a string", Token::Str("a string"));
    }

    #[test]
    #[cfg(feature = "serde_interop")]
    fn visit_unsupported_as_debug() {
        use serde_json::json;

        let v = json!({
            "id": 123,
            "name": "alice",
        });

        assert_visit(&v, Token::Args(&format!("{:?}", v)));
    }

    #[cfg(feature = "serde_interop")]
    mod serde_interop {
        use crate::*;
        use serde_test::{Token, assert_ser_tokens};
        use serde_json::json;

        // `&dyn ser::Serialize` should impl `Serialize`
        fn assert_visit(v: &dyn Visit, tokens: &[Token]) {
            assert_ser_tokens(&v, tokens);
        }

        #[test]
        fn visit_simple() {
            assert_visit(&1u8, &[Token::U8(1u8)]);
            assert_visit(&true, &[Token::Bool(true)]);
            assert_visit(&"a string", &[Token::Str("a string")]);
        }

        #[test]
        fn visit_complex() {
            let v = json!({
                "id": 123,
                "name": "alice",
            });

            assert_visit(&v, &[
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
