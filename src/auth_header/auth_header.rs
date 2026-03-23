use crate::date::utc_date_str;

use super::hmac_sha256;
use reqwest::header::{HeaderMap, IntoHeaderName, HeaderValue};
use chrono::{DateTime, TimeZone, Utc};
use crate::storage_request::StorageRequest;

use super::GET;

const MSDATE_KEY: &str = "x-ms-date";

const PROTOCOL: &str = "http";
const BLOB_SERVICE: &str = "azurite.local:10000";



pub struct AuthHeader<'a> {
    method: &'static str,
    store_account: Option<&'a str>,
    store_account_key: &'a str,
    path: Option<String>,
    datetime: Option<DateTime<Utc>>,
    // ms_headers: Vec<(String, String)>,
    // headers: Vec<(String, String)>,
    headermap: Option<HeaderMap>,
    query_params: Option<Vec<(String, String)>>,
    content_length: Option<usize>,
}

impl<'a> Default for AuthHeader<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> AuthHeader<'a> {
    pub fn new() -> Self {
        AuthHeader {
            method: GET,
            store_account: None,
            store_account_key: "",
            path: None,
            datetime: None,
            // ms_headers: Vec::new(),
            // headers: Vec::new(),
            headermap: None,
            query_params: None,
            content_length: None,
        }
    }

    /// set the datetime of the request. The DateTime<T> is translated to UTC time, as this is needed for the header.
    pub fn set_datetime<T>(mut self, dt: DateTime<T>) -> Self 
        where T: TimeZone {
        self.datetime = Some(dt.to_utc());
        self
    }

    /// get the string that needs to be signed to get the authorization-header.
    /// However, beware that header 'x-ms-date' still might be missing as that is added last-minute)
    /// used for internal purposes only. 
    fn get_string_to_sign(&self) -> String {
        // draw an array of 'x-ms-...'  headers and sort them on the key.
        let mut ms_headers: Vec<_> = self
            .headermap
            .as_ref()
            .expect("Headermap needs to be defined. Insert at least one headermap value via 'insert_header'.")
            .iter()
            .map(|(k, v)| (k.as_str(), v)) // only translate name, as value is not needed
            .filter(|(k, _)| k.starts_with("x-ms-"))
            .map(|(k, v)| (k, v.to_str().expect("x-ms- headers should only contain ascii-values. However binary value detected for key {k}")))
            .collect();
        ms_headers.sort_by(|a, b| a.0.cmp(&b.0));

        let ms_headers = ms_headers
            .iter()
            .map(|(k, v)| format!("{k}:{v}\n"))
            .collect::<Vec<_>>()
            .join("");
        let mut to_sign = format!(
            "{}\n\n\n{}\n\n\n\n\n\n\n\n\n{}/{}{}",
            self.method,
            self.content_length.map_or("".to_owned(), |x| x.to_string()),
            ms_headers,
            self.store_account
                .as_ref()
                .expect("use set_store_account to set the storage account"),
            self.path.as_ref().expect(
                "Use set_resources() to initialize the resource-path (including the initial '/')"
            )
        );
        // add resources to the to_sign string
        self.query_params
            .as_ref()
            .expect("Use set_query_params to add the query parameters. Add an empty vec if there are none.")
            .iter()
            .for_each(|p| {
                to_sign.push('\n');
                to_sign.push_str(&format!("{}:{}", p.0, p.1));
        });

        to_sign
    }

    fn get_shared_authorization(&self, to_sign: &str) -> String {
        let signed = hmac_sha256::get_hmac_b64(&self.store_account_key, &to_sign);

        let shared_auth = format!(
            "SharedKey {}:{}",
            self.store_account
                .as_ref()
                .expect("use set_store_account to set the storage account"),
            signed
        );
        shared_auth
    }

    pub fn set_method(mut self, method: &'static str) -> Self {
        self.method = method;
        self
    }

    pub fn set_store_account(mut self, store_account: &'a str, store_account_key: &'a str) -> Self {
        self.store_account = Some(store_account);
        self.store_account_key = store_account_key;
        self
    }

    pub fn set_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn set_content_length(mut self, len: usize) -> Self {
        self.content_length = Some(len);
        self
    }


    // set_query_paramters assumes the query parameters do not have redundant whitespace, are url-decoded and parameter-names are in lower-case.
    // In case of multiple parameter-values the values should be ordered!
    // source: https://learn.microsoft.com/en-us/rest/api/storageservices/authorize-with-shared-key#constructing-the-canonicalized-headers-string
    pub fn set_query_params(mut self, qp: &[(&str, &str)]) -> Self {
        let mut qp: Vec<_> = qp
            .iter()
            .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
            .collect();
        qp.sort_by(|a, b| a.0.cmp(&b.0));
        self.query_params = Some(qp);
        self
    }


    /// Insert the (key, value) as a header in the headermap, creating an empty headermap if none exists yet.
    pub fn insert_header<K>(mut self, key: K, value: HeaderValue) -> Self 
        where K: IntoHeaderName + ToString {
        // using to_string as
        assert!(key.to_string() != MSDATE_KEY, "Use the method 'self.set_date(...) to add a date to the headers." );
//        self.headermap = self.headermap.or(Some(HeaderMap::new()));

        self
            .headermap
            .get_or_insert(HeaderMap::new())
            .append(key, value);

        self
    }

    /// Build a 'StorageRequest' object based on the current input in the 'AuthHeader'.
    /// During the build phase the headermap is extended with a x-ms-date and an 'Authorization' header.
    pub fn build(mut self) -> StorageRequest {
        let url = format!("{PROTOCOL}://{}.{BLOB_SERVICE}{}", 
            self.store_account.as_ref().expect("Set storage account via 'set_storage_account' before building the request"), 
            self.path.as_ref().expect("Set path-parameters via 'set_path' before building the request."));

        // add missing headers needed to compute the shared-key
        {
            let hm = self
                .headermap
                .get_or_insert(HeaderMap::new());

            let datetime_str = utc_date_str(&self.datetime.unwrap_or(Utc::now()));
            hm.insert(MSDATE_KEY, HeaderValue::from_str(&datetime_str).unwrap());
            // add content length !!
        }
        let to_sign = self.get_string_to_sign();
        let auth_val = self.get_shared_authorization(&to_sign);

        let mut hm = self.headermap.expect("No headermap present. Insert value with 'insert_header'.");

        hm.insert("Authorization", auth_val.parse().unwrap());

        StorageRequest::new(url, self.query_params, to_sign, hm)
    }

}
