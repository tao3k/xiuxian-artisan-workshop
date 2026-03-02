/// Deserialize a single string or array of strings into `Vec<String>`.
pub(super) fn de_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or array of strings")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Vec<String>, E> {
            Ok(if value.is_empty() {
                Vec::new()
            } else {
                vec![value.to_string()]
            })
        }

        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Vec<String>, A::Error> {
            let mut out = Vec::new();
            while let Some(entry) = sequence.next_element::<String>()? {
                if !entry.is_empty() {
                    out.push(entry);
                }
            }
            Ok(out)
        }
    }

    deserializer.deserialize_any(StringOrVec)
}

/// Deserialize `Option` of string or array of strings into `Option<Vec<String>>`.
pub(super) fn de_opt_string_or_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct OptStringOrVec;

    impl<'de> Visitor<'de> for OptStringOrVec {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("optional string or array of strings")
        }

        fn visit_none<E: de::Error>(self) -> Result<Option<Vec<String>>, E> {
            Ok(None)
        }

        fn visit_some<D2: serde::Deserializer<'de>>(
            self,
            deserializer: D2,
        ) -> Result<Option<Vec<String>>, D2::Error> {
            let values = de_string_or_vec(deserializer)?;
            Ok(if values.is_empty() {
                None
            } else {
                Some(values)
            })
        }
    }

    deserializer.deserialize_option(OptStringOrVec)
}

pub(super) fn default_ref_doc_type() -> String {
    "reference".to_string()
}
