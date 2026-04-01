use reqwest::{blocking::{self, Response}, 
    Error, header::HeaderMap};
use crate::body::Body;
use crate::method::Method;
use std::ops::Deref;



pub struct StorageRequest<'c> {
    method: Method,
    url: String,
    query_params: Option<Vec<(String, String)>>,
    unsigned_authorization: String,
    headermap: Option<HeaderMap>, 
    body: Option<Body<'c>>
}


impl<'c> StorageRequest<'c> {

    pub fn new(method: Method, url: String, query_params: Option<Vec<(String, String)>>, unsigned_authorization: String, headermap: HeaderMap, body: Option<Body<'c>>) -> Self {
        let headermap = Some(headermap);
        StorageRequest{method, url, query_params, unsigned_authorization, headermap, body}
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }
    pub fn get_unsigned_authorization(&self) -> &str {
        &self.unsigned_authorization
    }

    pub fn get_headermap(&self) -> &HeaderMap {
        self.headermap.as_ref().expect("Headermap already has been extracted")
    }

    pub fn extract_headermap(&mut self) -> HeaderMap {
        self.headermap.take().expect("Headermap can be extracted only once")
    }

    pub fn get_query_params(&self) -> &Option<Vec<(String, String)>> {
        &self.query_params
    }

    /// this function will consume the 'body' as the body needs to be passed through, and that could extend minimal lifetime of the 'StorageRequest'.
    pub fn body_as_bytes(&self) -> Option<&'c [u8]> {
        // use and_then 
        match self.body {
            Some(b) => Some(b.as_bytes()),
            None => None
        }
    }

    pub fn exec_blocking(mut self) -> Result<Response, Error> {
        let client = blocking::Client::new();

        let client = match self.method {
            Method::Get => client.get(self.get_url()),   // why is to-owned needed?
            _ => panic!("method not recognized")

        };

        let client = client.headers(self.extract_headermap());

        let client = if let Some(query_params) = self.get_query_params() {
            client.query(query_params)
        } else {
            client
        };

        let client = if let Some(body) = self.body_as_bytes() {
            let bc = body.to_owned(); // is this an efficient solution.
            client.body(bc)
        } else {
            client
        };
        let res = client.send();

        res
    }

}
