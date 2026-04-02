/// The Body enum needs to be Copy otherwise it is difficult to unpack an Option<Body>
/// The body maintains a reference that later will be translated to a reqwest::Body or a reqwest::blocking::Body, which are two different structs.
/// Keeping it generic here, or producing a bytes::Bytes here allows this code to be generic whether for blocking or asynchronous requests.
#[derive(Clone,Copy)]
pub enum Body<'a> {
    Bytes(&'a[u8]),
    Text(&'a str)
}

impl<'a> Body<'a> {

    /// find the length of the body when translated to bytes.
    pub fn byte_len(&self) -> usize {
        match *self {
            Self::Bytes(b) => b.len(),
            Self::Text(s) => s.as_bytes().len()
        }
    }

    /// This function returns a reference to the byte-array of its contents. String-references are translated to byte-slices.
    /// Use the function str::from_utf8 to get back the underlying utf8 if it was a text-body.
    pub fn as_bytes(&self) -> &'a [u8] {
        match *self {
            Self::Bytes(b) => b,
            Self::Text(s) => s.as_bytes()
        }
    }
}
