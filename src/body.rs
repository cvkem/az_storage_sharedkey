use bytes::Bytes;


#[derive(Clone, Copy, PartialEq)]
enum BodyType {
    None,
    Text,
    Binary,
}
/// The Body enum needs to be Copy otherwise it is difficult to unpack an Option<Body>
/// The body maintains a reference that later will be translated to a reqwest::Body or a reqwest::blocking::Body, which are two different structs.
/// Keeping it generic here, or producing a bytes::Bytes here allows this code to be generic whether for blocking or asynchronous requests.
#[derive(Clone)]
pub struct Body {
    body_type: BodyType,
    data: Bytes
}

impl Body {

    pub fn from_bytes(data: Bytes) -> Self {
        Body{body_type: BodyType::Binary, data}
    }

    pub fn from_static(data: &'static [u8]) -> Self {
        Body{body_type: BodyType::Binary, data: Bytes::from_static(data)}
    }

    pub fn from_str(data: &str) -> Self {
        Body{body_type: BodyType::Text, data: Bytes::copy_from_slice(data.as_bytes())}
    }

    /// find the length of the body when translated to bytes.
    pub fn byte_len(&self) -> usize {
            self.data.len()
    }

    pub fn as_str(&self) -> &str {
        assert!(self.body_type == BodyType::Text, "Extraction as string is only allowed for Bodies generated from a string");
        std::str::from_utf8(self.data.as_ref()).expect("The input data should be valid UTF-8")
    }

    /// Extract the bytes, which destroys self
    pub fn into_bytes(self) -> Bytes {
        self.data
    }
    // /// This function returns a reference to the byte-array of its contents. String-references are translated to byte-slices.
    // /// Use the function str::from_utf8 to get back the underlying utf8 if it was a text-body.
    // pub fn as_bytes(&self) -> &'a [u8] {
    //     match *self {
    //         Self::Bytes(b) => b,
    //         Self::Text(s) => s.as_bytes(),
    //     }
    // }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_string_length() {
        let s = "\u{1F601}\u{1F602}\u{1F603}";
        println!("{s}");
        assert!(s.len() == s.as_bytes().len());
        assert!(s.len() == 12); // so the length of a string is its length as utf-8 bytes
        let s = "smiley: \u{1F602}\u{1F603}";
        assert!(s.len() == s.as_bytes().len());
        assert!(s.len() == 16); // so the length of a string is its length as utf-8 bytes
    }
}
