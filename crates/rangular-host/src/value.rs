use crate::event::EventPayload;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    List(Vec<Self>),
    Event(EventPayload),
    Unit,
}

impl Value {
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Str(s) => !s.is_empty(),
            Self::Num(n) => *n != 0.0,
            Self::List(items) => !items.is_empty(),
            Self::Event(EventPayload::Input { value }) => !value.is_empty(),
            Self::Event(EventPayload::Click { .. } | EventPayload::Error | EventPayload::Load) => {
                true
            }
            Self::Event(EventPayload::Custom(inner)) => inner.is_truthy(),
            Self::Unit => false,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            Self::Event(EventPayload::Input { value }) => Some(value.as_str()),
            Self::Event(EventPayload::Custom(inner)) => inner.as_str(),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_event(&self) -> Option<&EventPayload> {
        match self {
            Self::Event(payload) => Some(payload),
            _ => None,
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::Str(s.to_owned())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}
