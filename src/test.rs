//#[cfg(test)]
use crate::{
    auth_header::{self, AuthHeader},
    date::{utc_date_str_now, utc_date_str}};

use chrono::{Utc, TimeZone};
use reqwest::{
    blocking,
    header::{HeaderMap}
};


// default account and key based on:
//  https://docs.azure.cn/en-us/storage/common/storage-connect-azurite?tabs=blob-storage
//    Account name: devstoreaccount1
//    Account key: Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==

const TEST_STORE_ACCOUNT: &str = "devstoreaccount1";
const TEST_STORE_ACCOUNT_KEY_B64: &str = "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";


const PROTOCOL: &str = "http";
const BLOB_SERVICE: &str = "azurite.local:10000";
const CONTAINER: &str = "container";

const BLOB_NAME: &str = "blob_name";
const BLOB_CONTENT: &str = "Hello world!";



fn compare_strings(to_sign: &str, expect: &str) {
    println!("\nCompare the strings 'to_sign' and 'expected'");
        to_sign
            .chars()
            .zip(expect.chars())
            .enumerate()
            .for_each(|(idx, (t, s))|  println!("{idx}:  {t}  -  {s}     {}", if s!=t {"FAILED"} else {""}));
        println!("Lengths  to_sign: {}  expect: {}", to_sign.len(), expect.len());
}

fn check_to_sign_without_xmsdate(to_sign: &str, expect: &str) -> bool {
    use regex::Regex;

    let replacement = "x-ms-date:DDD, DD MMM YYYY HH:MM:SS GMT";
    let re = Regex::new(r"x-ms-date:\w{3}, \d{2} \w{3} \d{4} \d{2}:\d{2}:\d{2} GMT").unwrap();
    let to_sign = re.replace(to_sign, replacement);
    let expect = re.replace(expect, replacement);

    let matching = to_sign == expect;
    println!(" expected: {expect}\nmatch: {}", matching);
    if !matching {
        compare_strings(&to_sign, &expect);
    }
    matching
}

fn test_create_container() {

    println!("\nCreate container {CONTAINER} in store-account {TEST_STORE_ACCOUNT}");
    let client = blocking::Client::new();

    let path = format!("/{CONTAINER}");

    // create the date-string, such that headers and Authorization header (which contains signature of the header) use exactly same datetime.
    let utc_dt = utc_date_str_now();

    let mut headers = HeaderMap::new();

//    headers.insert(CONTENT_TYPE, "application/octet-stream".parse().unwrap());
    headers.insert("x-ms-date", utc_dt.parse().unwrap());
    headers.insert("x-ms-version", "2019-12-12".parse().unwrap());
    headers.insert("x-ms-blob-type", "PageBlob".parse().unwrap());

    let query_pars = [("restype", "container")];
    // first build auth-header witout autorization to be able to extract the 
    let auth_header = AuthHeader::new()
        .set_method(auth_header::PUT)
        .set_store_account(TEST_STORE_ACCOUNT.to_owned(), TEST_STORE_ACCOUNT_KEY_B64.to_owned())
        .set_path(path.to_owned())
        .set_query_params(&query_pars)
        .add_headermap(&headers);

    let to_sign = auth_header.get_string_to_sign();
    let auth_val = auth_header.get_shared_authorization();
    println!("string-to-sign: {to_sign}\nAuthorization: {auth_val}");
    assert!(check_to_sign_without_xmsdate(&to_sign, "PUT\n\n\n\n\n\n\n\n\n\n\n\nx-ms-blob-type:PageBlob\nx-ms-date:Sat, 14 Mar 2026 15:11:55 GMT\nx-ms-version:2019-12-12\n/devstoreaccount1/container\nrestype:container"));

    // now extend the header with the authorization (which contains a (partial) header signature).
    headers.insert("Authorization", auth_val.parse().unwrap());


    let create_container_url = format!("{PROTOCOL}://{TEST_STORE_ACCOUNT}.{BLOB_SERVICE}{path}");
    println!("URL: {}", create_container_url);
    let res = client
        .put(create_container_url)
        .headers(headers)
        .query(&query_pars)
        .send();

    println!("The PUT-response: {res:?}");

    assert!(res.is_ok(), "Write failed with result {res:?}");

}


fn test_create_block_blob() {
    let body_content = BLOB_CONTENT.as_bytes();

    println!("\nCreate blob '{BLOB_NAME}' in container '{CONTAINER}' in store-account '{TEST_STORE_ACCOUNT}'.");
    let client = blocking::Client::new();

    let path = format!("/{CONTAINER}/{BLOB_NAME}");

    // create the date-string, such that headers and Authorization header (which contains signature of the header) use exactly same datetime.
    let utc_dt = utc_date_str_now();

    let mut headers = HeaderMap::new();

//    headers.insert(CONTENT_TYPE, "application/octet-stream".parse().unwrap());  // is the default
    headers.insert("x-ms-date", utc_dt.parse().unwrap());
    headers.insert("x-ms-version", "2019-12-12".parse().unwrap());
    headers.insert("x-ms-blob-type", "BlockBlob".parse().unwrap());
    headers.insert("x-ms-blob-content-length", "512".parse().unwrap());  // required for pageblobs. Should be multiple of 512

    // first build auth-header witout autorization to be able to extract the 
    let auth_header = AuthHeader::new()
        .set_method(auth_header::PUT)
        .set_store_account(TEST_STORE_ACCOUNT.to_owned(), TEST_STORE_ACCOUNT_KEY_B64.to_owned())
        .set_path(path.to_owned())
        .add_headermap(&headers)
        .set_content_length(body_content.len())
        .set_query_params(&[]);


    let to_sign = auth_header.get_string_to_sign();
    let auth_val = auth_header.get_shared_authorization();
    println!("string-to-sign: {to_sign}\nAuthorization: {auth_val}");
    assert!(check_to_sign_without_xmsdate(&to_sign, "PUT\n\n\n12\n\n\n\n\n\n\n\n\nx-ms-blob-content-length:512\nx-ms-blob-type:BlockBlob\nx-ms-date:Sat, 14 Mar 2026 15:11:55 GMT\nx-ms-version:2019-12-12\n/devstoreaccount1/container/blob_name"));

    // now extend the header with the authorization (which contains a (partial) header signature).
    headers.insert("Authorization", auth_val.parse().unwrap());


    let create_container_url = format!("{PROTOCOL}://{TEST_STORE_ACCOUNT}.{BLOB_SERVICE}{path}");
    println!("URL: {}", create_container_url);
    let res = client
        .put(create_container_url)
        .headers(headers)
        .body(body_content)
        .send();

    println!("The PUT-response: {res:?}");

    assert!(res.is_ok(), "Write failed with result {res:?}");

}

fn test_get_block_blob() {
    let body_content = BLOB_CONTENT.as_bytes();

    println!("\nGet blob '{BLOB_NAME}' in container '{CONTAINER}' in store-account '{TEST_STORE_ACCOUNT}'.");
    let client = blocking::Client::new();

    let path = format!("/{CONTAINER}/{BLOB_NAME}");

    // create the date-string, such that headers and Authorization header (which contains signature of the header) use exactly same datetime.
    let utc_dt = utc_date_str_now();

    let mut headers = HeaderMap::new();

//    headers.insert(CONTENT_TYPE, "application/octet-stream".parse().unwrap());  // is the default
    headers.insert("x-ms-date", utc_dt.parse().unwrap());
    headers.insert("x-ms-version", "2019-12-12".parse().unwrap());

    // first build auth-header witout autorization to be able to extract the 
    let auth_header = AuthHeader::new()
        .set_method(auth_header::GET)
        .set_store_account(TEST_STORE_ACCOUNT.to_owned(), TEST_STORE_ACCOUNT_KEY_B64.to_owned())
        .set_path(path.to_owned())
        .set_headermap(&headers)
        .set_query_params(&[]);


    let to_sign = auth_header.get_string_to_sign();
    let auth_val = auth_header.get_shared_authorization();
    println!("string-to-sign: {to_sign}\nAuthorization: {auth_val}");
    assert!(check_to_sign_without_xmsdate(&to_sign, "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Sat, 14 Mar 2026 15:11:55 GMT\nx-ms-version:2019-12-12\n/devstoreaccount1/container/blob_name"));

    // now extend the header with the authorization (which contains a (partial) header signature).
    headers.insert("Authorization", auth_val.parse().unwrap());


    let get_container_url = format!("{PROTOCOL}://{TEST_STORE_ACCOUNT}.{BLOB_SERVICE}{path}");
    println!("URL: {}", get_container_url);
    let res = client
        .get(get_container_url)
        .headers(headers)
        .send();

    println!("The GET-response: {res:?}");

    assert!(res.is_ok(), "Get failed with result {res:?}");

    let data = res.expect("Expected response-success").bytes().expect("Data as bytes in reponse");
    let s = String::from_utf8_lossy(&data);

    println!("Retrieved data: {s}");

}


#[test]
fn test_authorization() {
    // Building up next request
    // GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Fri, 26 Jun 2015 23:39:12 GMT\nx-ms-version:2015-02-21\n/myaccount/mycontainer\ncomp:metadata\nrestype:container\ntimeout:20
    // Authorization: SharedKey myaccount:ctzMq410TV3wS7upTBcunJTDLEJwMAZuFPfr0mrrA08=

    // the default storage account
    // DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;QueueEndpoint=http://127.0.0.1:10001/devstoreaccount1;TableEndpoint=http://127.0.0.1:10002/devstoreaccount1;DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;QueueEndpoint=http://127.0.0.1:10001/devstoreaccount1;TableEndpoint=http://127.0.0.1:10002/devstoreaccount1;DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;QueueEndpoint=http://127.0.0.1:10001/devstoreaccount1;TableEndpoint=http://127.0.0.1:10002/devstoreaccount1;


    let dt = Utc.with_ymd_and_hms(2015, 6, 26, 23, 39, 12).unwrap();
    println!("The date = {dt:?}");
    let dts = utc_date_str(&dt);
    println!("\ttranslates to x-ms-date: {dts}");

    let mut headers = HeaderMap::new();
    headers.insert("x-ms-date", dts.parse().unwrap());
    headers.insert("x-ms-version", "2015-02-21".parse().unwrap());

    let auth_header = AuthHeader::new()
        .set_method(auth_header::GET) 
        .set_store_account("myaccount".to_owned(), TEST_STORE_ACCOUNT_KEY_B64.to_owned())
        .set_path("/mycontainer".to_owned())
        .add_headermap(&headers)
        //.set_datetime(&dt)  // Better to use UTC, but TZ should be dropped anyway
        .set_query_params(&[("comp", "metadata"), ("restype", "container"), ("timeout","20")]);

    let to_sign = auth_header.get_string_to_sign();
                // FOR Debugging only
    println!("to-sign = {}", to_sign);
    assert!(check_to_sign_without_xmsdate(&to_sign, "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Fri, 26 Jun 2015 23:39:12 GMT\nx-ms-version:2015-02-21\n/myaccount/mycontainer\ncomp:metadata\nrestype:container\ntimeout:20"));

    println!("The full authorization header:\nAuthorization: {}", auth_header.get_shared_authorization());
    println!("Expected:                     \nAuthorization: SharedKey myaccount:ctzMq410TV3wS7upTBcunJTDLEJwMAZuFPfr0mrrA08=")
}

#[test]
fn run_tests_in_sequence() {

    test_create_container();

    test_create_block_blob();

    test_get_block_blob();
}
