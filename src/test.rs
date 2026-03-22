//#[cfg(test)]
use crate::{
    auth_header::{self, AuthHeader}
};

use chrono::{TimeZone, Utc};
use reqwest::{blocking};

// default account and key based on:
//  https://docs.azure.cn/en-us/storage/common/storage-connect-azurite?tabs=blob-storage
//    Account name: devstoreaccount1
//    Account key: Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==

const TEST_STORE_ACCOUNT: &str = "devstoreaccount1";
const TEST_STORE_ACCOUNT_KEY_B64: &str =
    "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";

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
        .for_each(|(idx, (t, s))| {
            println!(
                "{idx}:  {t}  -  {s}     {}",
                if s != t { "FAILED" } else { "" }
            )
        });
    println!(
        "Lengths  to_sign: {}  expect: {}",
        to_sign.len(),
        expect.len()
    );
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

    let query_pars = [("restype", "container")];
    // first build auth-header witout autorization to be able to extract the
    let auth_header = AuthHeader::new()
        .set_method(auth_header::PUT)
        .set_store_account(
            TEST_STORE_ACCOUNT,
            TEST_STORE_ACCOUNT_KEY_B64,
        )
        .set_path(path.to_owned())
        .set_query_params(&query_pars)
        .insert_header("x-ms-version", "2019-12-12".parse().unwrap());

    let to_sign = auth_header.get_string_to_sign();
    let auth_val = auth_header.get_shared_authorization();
    println!("string-to-sign: {to_sign}\nAuthorization: {auth_val}");
    assert!(check_to_sign_without_xmsdate(
        &to_sign,
        "PUT\n\n\n\n\n\n\n\n\n\n\n\nx-ms-version:2019-12-12\n/devstoreaccount1/container\nrestype:container"
//        "PUT\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Sat, 14 Mar 2026 15:11:55 GMT\nx-ms-version:2019-12-12\n/devstoreaccount1/container\nrestype:container"
    ));

    let headers = auth_header.get_headermap();

    let create_container_url = format!("{PROTOCOL}://{TEST_STORE_ACCOUNT}.{BLOB_SERVICE}{path}");
    println!("URL: {}", create_container_url);
    let res = client
        .put(create_container_url)
        .headers(headers)
        .query(&query_pars)
        .send();

    println!("The PUT-response: {res:?}");
    let status = res
        .expect("Create container failed with result {res:?}")
        .status();
    assert!(
        status == reqwest::StatusCode::CREATED,
        "Expected status 201 CREATED, but observed http-status: {status}"
    );
}

fn test_create_block_blob() {
    let body_content = BLOB_CONTENT.as_bytes();

    println!(
        "\nCreate blob '{BLOB_NAME}' in container '{CONTAINER}' in store-account '{TEST_STORE_ACCOUNT}'."
    );
    let client = blocking::Client::new();

    let path = format!("/{CONTAINER}/{BLOB_NAME}");

    // I can also pass a reference to a lokal string as this is guaranteed to exit long enough
    let t_a = TEST_STORE_ACCOUNT.to_owned();
    // first build auth-header witout autorization to be able to extract the
    let auth_header = AuthHeader::new()
        .set_method(auth_header::PUT)
        .set_store_account(
            &t_a,
            TEST_STORE_ACCOUNT_KEY_B64,
        )
        .set_path(path.to_owned())
        .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
        .insert_header("x-ms-blob-type", "BlockBlob".parse().unwrap())
        //.insert_header("x-ms-blob-content-length", "512".parse().unwrap()); // required for pageblobs. Should be multiple of 512
        .set_content_length(body_content.len())
        .set_query_params(&[]);

    let to_sign = auth_header.get_string_to_sign();
    let auth_val = auth_header.get_shared_authorization();
    println!("string-to-sign: {to_sign}\nAuthorization: {auth_val}");
    assert!(check_to_sign_without_xmsdate(
        &to_sign,
        // Warning: x-ms-date is missing
        "PUT\n\n\n12\n\n\n\n\n\n\n\n\nx-ms-blob-type:BlockBlob\nx-ms-version:2019-12-12\n/devstoreaccount1/container/blob_name"
//        "PUT\n\n\n12\n\n\n\n\n\n\n\n\nx-ms-blob-content-length:512\nx-ms-blob-type:BlockBlob\nx-ms-date:Sat, 14 Mar 2026 15:11:55 GMT\nx-ms-version:2019-12-12\n/devstoreaccount1/container/blob_name"
    ));

    let headers = auth_header.get_headermap();

    let create_container_url = format!("{PROTOCOL}://{TEST_STORE_ACCOUNT}.{BLOB_SERVICE}{path}");
    println!("URL: {}", create_container_url);
    let res = client
        .put(create_container_url)
        .headers(headers)
        .body(body_content)
        .send();

    println!("The PUT-response: {res:?}");

    let status = res.expect("Write blob failed with result {res:?}").status();
    assert!(
        status == reqwest::StatusCode::CREATED,
        "Expected status 201 CREATED, but observed http-status: {status}"
    );
}

fn test_get_block_blob() {
    println!(
        "\nGet blob '{BLOB_NAME}' in container '{CONTAINER}' in store-account '{TEST_STORE_ACCOUNT}'."
    );
    let client = blocking::Client::new();

    let path = format!("/{CONTAINER}/{BLOB_NAME}");

    let auth_header = AuthHeader::new()
        .set_method(auth_header::GET)
        .set_store_account(
            TEST_STORE_ACCOUNT,
            TEST_STORE_ACCOUNT_KEY_B64,
        )
        .set_path(path.to_owned())
        .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
        .set_query_params(&[]);

    let to_sign = auth_header.get_string_to_sign();
    let auth_val = auth_header.get_shared_authorization();
    println!("string-to-sign: {to_sign}\nAuthorization: {auth_val}");
    assert!(check_to_sign_without_xmsdate(
        &to_sign,
        "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-version:2019-12-12\n/devstoreaccount1/container/blob_name"
//        "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Sat, 14 Mar 2026 15:11:55 GMT\nx-ms-version:2019-12-12\n/devstoreaccount1/container/blob_name"
    ));

    // now extend the header with the authorization (which contains a (partial) header signature).
    //headers.insert("Authorization", auth_val.parse().unwrap());

    let headers = auth_header.get_headermap();

    let get_container_url = format!("{PROTOCOL}://{TEST_STORE_ACCOUNT}.{BLOB_SERVICE}{path}");
    println!("URL: {}", get_container_url);
    let res = client.get(get_container_url).headers(headers).send();

    println!("The GET-response: {res:?}");

    let status = res
        .as_ref()
        .expect("Get blob failed with result {res:?}")
        .status();
    assert!(
        status == reqwest::StatusCode::OK,
        "Expected status 201 CREATED, but observed http-status: {status}"
    );

    let data = res
        .expect("Expected response-success")
        .bytes()
        .expect("Data as bytes in reponse");
    let s = String::from_utf8_lossy(&data);

    println!("Retrieved data: {s}");
}


#[test]
fn run_tests_in_sequence() {
    test_create_container();

    test_create_block_blob();

    test_get_block_blob();
}
