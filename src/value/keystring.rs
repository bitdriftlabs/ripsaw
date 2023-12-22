use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

use bytes::Bytes;
use serde::Deserialize;
use serde::Serialize;
use std::hash::Hash;

/// The key type value. This is a simple zero-overhead wrapper set up to make it explicit that
/// object keys are read-only and their underlying type is opaque and may change for efficiency.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
//fixfix#[cfg_attr(any(test, feature = "proptest"), derive(proptest_derive::Arbitrary))]
//fixfix#[serde(transparent)]
pub struct KeyString(Bytes);

impl KeyString {
    /// Convert the key into a boxed slice of bytes (`u8`).
    #[inline]
    #[must_use]
    pub fn into_bytes(self) -> Box<[u8]> {
        self.0.to_vec().into()
    }

    /// Convert the key to the backing bytes.
    pub fn to_bytes(&self) -> Bytes {
        self.0.clone()
    }

    /// Is this string empty?
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the length of the contained key.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Return a reference to the contained string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        // Must be a valid string.
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl Hash for KeyString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Strings hash differently from bytes so make sure the implementations below line up.
        self.as_str().hash(state);
    }
}

impl Serialize for KeyString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'a> Deserialize<'a> for KeyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let string = String::deserialize(deserializer)?;
        Ok(string.into())
    }
}

impl Display for KeyString {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(fmt)
    }
}

impl AsRef<str> for KeyString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::ops::Deref for KeyString {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for KeyString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for KeyString {
    fn eq(&self, that: &str) -> bool {
        self.as_str()[..].eq(that)
    }
}

impl From<&str> for KeyString {
    fn from(s: &str) -> Self {
        Self(s.to_string().into())
    }
}

impl From<String> for KeyString {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<Cow<'_, str>> for KeyString {
    fn from(s: Cow<'_, str>) -> Self {
        Self(s.into_owned().into())
    }
}

impl From<KeyString> for String {
    fn from(s: KeyString) -> Self {
        s.as_str().to_string()
    }
}

#[cfg(any(test, feature = "proptest"))]
impl proptest::prelude::Arbitrary for KeyString {
    type Parameters = proptest::string::StringParam;
    type Strategy = proptest::prelude::BoxedStrategy<Self>;

    fn arbitrary_with(parameters: Self::Parameters) -> Self::Strategy {
        use proptest::prelude::*;

        String::arbitrary_with(parameters)
            .prop_map(Self::from)
            .boxed()
    }
}

#[cfg(any(test, feature = "arbitrary"))]
impl quickcheck::Arbitrary for KeyString {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        String::arbitrary(g).into()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let s = self.as_str().to_string();
        Box::new(s.shrink().map(Into::into))
    }
}

#[cfg(any(test, feature = "lua"))]
mod lua {
    use mlua::prelude::LuaResult;
    use mlua::{FromLua, IntoLua, Lua, Value as LuaValue};

    use super::KeyString;

    impl FromLua for KeyString {
        fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<Self> {
            String::from_lua(value, lua).map(Self::from)
        }
    }

    impl IntoLua for KeyString {
        fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
            self.as_str().into_lua(lua)
        }
    }
}
