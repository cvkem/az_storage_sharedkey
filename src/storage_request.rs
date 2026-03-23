use reqwest::header::HeaderMap;



pub struct StorageRequest {
    url: String,
    query_params: Option<Vec<(String, String)>>,
    unsigned_authorization: String,
    headermap: Option<HeaderMap>, 
}


impl StorageRequest {

    pub fn new(url: String, query_params: Option<Vec<(String, String)>>, unsigned_authorization: String, headermap: HeaderMap) -> Self {
        let headermap = Some(headermap);
        StorageRequest{url, query_params, unsigned_authorization, headermap}
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

}
