use crate::date::utc_date_str;

use super::hmac_sha256;
use reqwest::header::{HeaderMap, IntoHeaderName, HeaderValue};
use chrono::{DateTime, TimeZone, Utc};
use urlencoding::decode;

use super::GET;

const MSDATE_kEY: &str = "x-ms-date";

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
    content_length: String,
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
            content_length: "".to_owned(),
        }
    }

    pub fn set_datetime<T>(mut self, dt: DateTime<T>) -> Self 
        where T: TimeZone {
        self.datetime = Some(dt.to_utc());
        self
    }

    /// get the string that needs to be signed to get the authorization-header.
    /// However, beware that header 'x-ms-date' still might be missing as that is added last-minute)
    pub fn get_string_to_sign(&self) -> String {
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
            self.content_length,
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

    pub fn get_shared_authorization(&self) -> String {
        let to_sign = self.get_string_to_sign();

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
        self.content_length = format!("{len}");
        self
    }

    // pub fn set_datetime<T: TimeZone>(mut self, dt: &DateTime<T>) -> Self {
    //     self.utc_date_str = Some(utc_date_str(dt));
    //     self
    // }

    // // assume the date_str
    // pub fn set_datetime_str(mut self, utc_date_str: String) -> Self {
    //     self.utc_date_str = Some(utc_date_str);
    //     self
    // }

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

    // // collect the x-ms- headers, order them, url-decode them  and add these to the vectors with ms_headers and headers based on the key-prefix.
    // pub fn add_headermap(mut self, headers: &HeaderMap) -> Self {
    //     headers
    //         .iter()
    //         .map(|(key, val)| {
    //             (
    //                 decode(key.as_str()).expect("UTF-8 key").into_owned(),
    //                 decode(val.to_str().unwrap())
    //                     .expect("UFT-8 value")
    //                     .into_owned(),
    //             )
    //         })
    //         .map(|(k, v)| (k.to_string(), v.to_string()))
    //         .for_each(|kv| {
    //             if kv.0.as_str().starts_with("x-ms-") {
    //                 self.ms_headers.push(kv);
    //             } else {
    //                 self.headers.push(kv);
    //             }
    //         });
    //     self
    // }

    /// Insert the (key, value) as a header in the headermap, creating an empty headermap if none exists yet.
    pub fn insert_header<K>(mut self, key: K, value: HeaderValue) -> Self 
        where K: IntoHeaderName + ToString {
        // using to_string as
        assert!(key.to_string() != MSDATE_kEY, "Use the method 'self.set_date(...) to add a date to the headers." );
//        self.headermap = self.headermap.or(Some(HeaderMap::new()));

        self
            .headermap
            .get_or_insert(HeaderMap::new())
            .append(key, value);

        self
    }

    /// get the existing headermap and extend it with the with an x-ms-date and an Autorization field.
    pub fn get_headermap(mut self) -> HeaderMap {
        // add missing headers needed to compute the shared-key
        {
            let hm = self
                .headermap
                .get_or_insert(HeaderMap::new());

            let datetime_str = utc_date_str(&self.datetime.unwrap_or(Utc::now()));
            hm.insert(MSDATE_kEY, HeaderValue::from_str(&datetime_str).unwrap());
            // add content length !!
        }
        let auth_val = self.get_shared_authorization();

        let mut hm = self.headermap.expect("No headermap present. Insert value with 'insert_header'.");

        hm.insert("Authorization", auth_val.parse().unwrap());

        hm
    }

    // // clearn the collected headers and replace them with headers from the headersmap
    // pub fn set_headermap(mut self, headers: &HeaderMap) -> Self {
    //     self.ms_headers.clear();
    //     self.headers.clear();

    //     self.add_headermap(headers)
    // }

    // // add the header to the righ queue
    // pub fn add_header(mut self, k: String, v: String) -> Self {
    //     if k.starts_with("x-ms-") {
    //         self.ms_headers.push((k, v))
    //     } else {
    //         self.headers.push((k, v))
    //     }
    //     self
    // }

    // pub fn get_headermap(&self) -> HeaderMap {
    //     let mut hm = HeaderMap::new();

    //     self.headers.iter().for_each(|(k, v)| {
    //         let _ = hm.append(
    //             HeaderName::from_bytes(k.as_bytes()).unwrap(),
    //             v.to_owned().parse().unwrap(),
    //         );
    //     });
    //     hm
    // }
}
