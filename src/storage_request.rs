use crate::body::Body;
use crate::method::Method;
use reqwest::{
    Error,
    blocking::{self, Response},
    header::HeaderMap,
};

pub struct StorageRequest {
    method: Method,
    url: String,
    query_params: Option<Vec<(String, String)>>,
    unsigned_authorization: String,
    headermap: Option<HeaderMap>,
    body: Option<Body>,
}

impl StorageRequest {
    pub fn new(
        method: Method,
        url: String,
        query_params: Option<Vec<(String, String)>>,
        unsigned_authorization: String,
        headermap: HeaderMap,
        body: Option<Body>,
    ) -> Self {
        let headermap = Some(headermap);
        StorageRequest {
            method,
            url,
            query_params,
            unsigned_authorization,
            headermap,
            body,
        }
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }
    pub fn get_unsigned_authorization(&self) -> &str {
        &self.unsigned_authorization
    }

    pub fn get_headermap(&self) -> &HeaderMap {
        self.headermap
            .as_ref()
            .expect("Headermap already has been extracted")
    }

    pub fn extract_headermap(&mut self) -> HeaderMap {
        self.headermap
            .take()
            .expect("Headermap can be extracted only once")
    }

    pub fn get_query_params(&self) -> &Option<Vec<(String, String)>> {
        &self.query_params
    }


    pub fn exec_blocking(mut self) -> Result<Response, Error> {
        let client = blocking::Client::new();

        let client = match self.method {
            Method::Get => client.get(self.get_url()),
            Method::Head => client.head(self.get_url()),
            Method::Post => client.post(self.get_url()),
            Method::Put => client.put(self.get_url()),
            Method::Delete => client.delete(self.get_url()),
            // Method::Connect => client.connect(self.get_url()),
            // Method::Options => client.options(self.get_url()),
            // Method::Trace => client.trace(self.get_url()),
            _ => panic!("method '{}' not available in reqwest", self.method),
        };

        let client = client.headers(self.extract_headermap());

        let client = if let Some(query_params) = self.get_query_params() {
            client.query(query_params)
        } else {
            client
        };

        let client = if let Some(body) = self.body {
            client.body(body.into_bytes())
        } else {
            client
        };
        let res = client.send();

        res
    }
}
