use crate::date::utc_date_str;

use super::hmac_sha256;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, IntoHeaderName};
use chrono::{DateTime, TimeZone, Utc};
use crate::storage_request::StorageRequest;
use crate::body::Body;
use super::GET;


const PROTOCOL: &str = "http";


#[derive(Clone)]
pub struct AuthHeader<'a,'b,'c> {
    method: &'static str,
    store_account: Option<&'a str>,
    store_account_key: &'a str,
    dns_suffix: Option<&'b str>,
    path: Option<String>,
    datetime: Option<DateTime<Utc>>,
    headermap: Option<HeaderMap>,
    query_params: Option<Vec<(String, String)>>,
    content_length: usize,
    body: Option<Body<'c>>
}

impl<'a,'b,'c> Default for AuthHeader<'a,'b,'c> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a,'b,'c> AuthHeader<'a,'b,'c> {
    pub fn new() -> Self {
        AuthHeader {
            method: GET,
            store_account: None,
            store_account_key: "",
            dns_suffix: None,
            path: None,
            datetime: None,
            headermap: None,
            query_params: None,
            content_length: 0,
            body: None
        }
    }

    /// set the datetime of the request. The DateTime<T> is translated to UTC time, as this is needed for the header.
    pub fn set_datetime<T>(mut self, dt: DateTime<T>) -> Self 
        where T: TimeZone {
        self.datetime = Some(dt.to_utc());
        self
    }

    /// Headers might contain multiple values however, for as this function can only return one header-value it panics when multiple headervalues are present.
    fn get_header_value(&self, key: &HeaderName) -> &str {
        let mut values = self.headermap.as_ref().expect("headmap should have at least one key").get_all(key).iter();
        match (values.next(), values.next()) {
            (None, None) => "",
            (Some(value), None) => value.to_str().expect("value of key '{key}' should be string. Received {value:?}"),
            _ => {
                assert!(false, "Key '{key}' has more than one value");
                ""
            }
        }
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

        let ms_header_str = ms_headers
            .iter()
            .map(|(k, v)| format!("{k}:{v}\n"))
            .collect::<Vec<_>>()
            .join("");

        // source of specifications: https://learn.microsoft.com/en-us/rest/api/storageservices/authorize-with-shared-key#blob-queue-and-file-services-shared-key-authorization
        // GET\n /*HTTP Verb*/  
        // \n    /*Content-Encoding*/  
        // \n    /*Content-Language*/  
        // \n    /*Content-Length (empty string when zero)*/  
        // \n    /*Content-MD5*/  
        // \n    /*Content-Type*/  
        // \n    /*Date*/  
        // \n    /*If-Modified-Since */  
        // \n    /*If-Match*/  
        // \n    /*If-None-Match*/  
        // \n    /*If-Unmodified-Since*/  
        // \n    /*Range*/  
        // x-ms-date:Fri, 26 Jun 2015 23:39:12 GMT\nx-ms-version:2015-02-21\n    /*CanonicalizedHeaders*/  
        // /myaccount /mycontainer\ncomp:metadata\nrestype:container\ntimeout:20    /*CanonicalizedResource*/ 

            let mut to_sign = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n\n\n\n\n\n\n{}/{}{}",
            self.method,
            self.get_header_value(&reqwest::header::CONTENT_ENCODING),
            self.get_header_value(&reqwest::header::CONTENT_LANGUAGE),            
            if self.content_length > 0 { self.content_length.to_string()} else { "".to_owned() },
            self.get_header_value(&HeaderName::from_static("content_md5")),            
            self.get_header_value(&reqwest::header::CONTENT_TYPE),            
            ms_header_str,
            self.store_account
                .as_ref()
                .expect("use set_store_account to set the storage account"),
            self.get_path()
        );
        // add resources to the to_sign string
        self.get_query_params()
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

    pub fn set_dns_suffix(mut self, dns_suffix: &'b str) -> Self {
        self.dns_suffix = Some(dns_suffix);
        self
    }

    pub fn set_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn get_path(&self) -> &str {
        self.path.as_ref().map(|v| v.as_str()).unwrap_or("/")
    }

    /// When adding a body via 'set_body()' the content length is automatically added. So only use this function if you do want to set content_lemgth without setting a body.
    pub fn set_content_length_without_body(mut self, len: usize) -> Self {
        assert!(self.content_length == 0, "Can not set content-lenght twice. set_body already sets the content length.");
        self.content_length = len;
        self
    }

    pub fn set_body(mut self, body: Body<'c>) -> Self{
        assert!(self.body.is_none(), "Body has already been set. can not set body twice");
        // set content length to the byte-length of the body, even if it is a string.
        self.body = Some(body);
        self.set_content_length_without_body(body.byte_len())
    }

    // set_query_parameters assumes the query parameters do not have redundant whitespace, are url-decoded and parameter-names are in lower-case.
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

    pub fn get_query_params(&self) -> &[(String,String)]{
        self.query_params
            .as_ref()
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }


    /// Insert the (key, value) as a header in the headermap, creating an empty headermap if none exists yet.
    pub fn insert_header<K>(mut self, key: K, value: HeaderValue) -> Self 
        where K: IntoHeaderName + ToString {
        // using to_string as
        {
            let key_str = key.to_string();
            assert!(key_str != super::MSDATE_KEY, "Use the method 'self.set_date(...)' to add a date-headers.");
            assert!(key_str != reqwest::header::CONTENT_LENGTH.as_str(), "Use the method 'self.set_body(...) which does calculate and add a content-length to the headers." );
        }
//        self.headermap = self.headermap.or(Some(HeaderMap::new()));

        self
            .headermap
            .get_or_insert(HeaderMap::new())
            .append(key, value);

        self
    }


    /// Build a 'StorageRequest' object based on the current input in the 'AuthHeader'.
    /// During the build phase the headermap is extended with a x-ms-date and an 'Authorization' header.
    pub fn build(mut self) -> StorageRequest<'c> {
        let url = format!("{PROTOCOL}://{}.{}{}", 
            self.store_account.as_ref().expect("Set storage-account via 'set_storage_account' before building the request"),
            self.dns_suffix.as_ref().expect("Set the dns-suffix via 'set_dns_suffix' before building the request"),
            self.get_path()); // emtpy path translates to '/'.  //  expect("Set path-parameters via 'set_path' before building the request."));

        // add missing headers needed to compute the shared-key
        {
            let hm = self
                .headermap
                .get_or_insert(HeaderMap::new());

            let datetime_str = utc_date_str(&self.datetime.unwrap_or(Utc::now()));
            hm.insert(super::MSDATE_KEY, HeaderValue::from_str(&datetime_str).unwrap());

            if self.content_length > 0 {
                 hm.insert(reqwest::header::CONTENT_LENGTH , self.content_length.to_string().parse().unwrap());
            } 
            // add content length !!
        }
        let to_sign = self.get_string_to_sign();
        let auth_val = self.get_shared_authorization(&to_sign);

        let mut hm = self.headermap.expect("No headermap present. Insert value with 'insert_header'.");

        hm.insert(reqwest::header::AUTHORIZATION , auth_val.parse().unwrap());

        StorageRequest::new(url, self.query_params, to_sign, hm, self.body)
    }

}
