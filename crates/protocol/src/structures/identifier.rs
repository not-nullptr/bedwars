use std::{borrow::Cow, fmt::Display, hash::Hash};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub namespace: Cow<'static, str>,
    pub value: Cow<'static, str>,
}

impl Identifier {
    pub fn new(value: impl Into<Cow<'static, str>>) -> Self {
        Self {
            namespace: "minecraft".into(),
            value: value.into(),
        }
    }

    pub fn with_namespace(
        namespace: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            value: value.into(),
        }
    }

    pub const fn const_new(value: &'static str) -> Self {
        Self {
            namespace: Cow::Borrowed("minecraft"),
            value: Cow::Borrowed(value),
        }
    }

    pub const fn const_with_namespace(namespace: &'static str, value: &'static str) -> Self {
        Self {
            namespace: Cow::Borrowed(namespace),
            value: Cow::Borrowed(value),
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.value)
    }
}

impl Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        ':'.hash(state);
        self.value.hash(state);
    }
}

impl Serialize for Identifier {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}
