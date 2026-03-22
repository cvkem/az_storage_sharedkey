use super::hmac_sha256;
use reqwest::header::{HeaderMap, HeaderName};
use urlencoding::decode;

use super::GET;
pub struct AuthHeader {
    method: &'static str,
    store_account: Option<String>,
    store_account_key: String,
    path: Option<String>,
    ms_headers: Vec<(String, String)>,
    headers: Vec<(String, String)>,
    query_params: Option<Vec<(String, String)>>,
    content_length: String,
}

impl Default for AuthHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthHeader {
    pub fn new() -> Self {
        AuthHeader {
            method: GET,
            store_account: None,
            store_account_key: String::new(),
            path: None,
            ms_headers: Vec::new(),
            headers: Vec::new(),
            query_params: None,
            content_length: "".to_owned(),
        }
    }

    pub fn get_string_to_sign(&self) -> String {
        // draw an array of slices that can be ordered, without changing the original (as it is not mutable) and next sort it.
        let mut ms_headers: Vec<_> = self.ms_headers.iter().collect();
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

    pub fn set_store_account(mut self, store_account: String, store_account_key: String) -> Self {
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

    // assuming the query paramters do not have redundant whitespace, are url-decoded and parameter-names are in lower-case.
    // In case of multiple parameter-values the values should be ordered!
    // source: https://learn.microsoft.com/en-us/rest/api/storageservices/authorize-with-shared-key#constructing-the-canonicalized-headers-string
    // TODO: add cleansing (and split parameter and value)
    pub fn set_query_params(mut self, qp: &[(&str, &str)]) -> Self {
        let mut qp: Vec<_> = qp
            .iter()
            .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
            .collect();
        qp.sort_by(|a, b| a.0.cmp(&b.0));
        self.query_params = Some(qp);
        self
    }

    // collect the x-ms- headers, order them, url-decode them  and add these to the vectors with ms_headers and headers based on the key-prefix.
    pub fn add_headermap(mut self, headers: &HeaderMap) -> Self {
        headers
            .iter()
            .map(|(key, val)| {
                (
                    decode(key.as_str()).expect("UTF-8 key").into_owned(),
                    decode(val.to_str().unwrap())
                        .expect("UFT-8 value")
                        .into_owned(),
                )
            })
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .for_each(|kv| {
                if kv.0.as_str().starts_with("x-ms-") {
                    self.ms_headers.push(kv);
                } else {
                    self.headers.push(kv);
                }
            });
        self
    }

    // clearn the collected headers and replace them with headers from the headersmap
    pub fn set_headermap(mut self, headers: &HeaderMap) -> Self {
        self.ms_headers.clear();
        self.headers.clear();

        self.add_headermap(headers)
    }

    // add the header to the righ queue
    pub fn add_header(mut self, k: String, v: String) -> Self {
        if k.starts_with("x-ms-") {
            self.ms_headers.push((k, v))
        } else {
            self.headers.push((k, v))
        }
        self
    }

    pub fn get_headermap(&self) -> HeaderMap {
        let mut hm = HeaderMap::new();

        self.headers.iter().for_each(|(k, v)| {
            let _ = hm.append(
                HeaderName::from_bytes(k.as_bytes()).unwrap(),
                v.to_owned().parse().unwrap(),
            );
        });
        hm
    }
}
