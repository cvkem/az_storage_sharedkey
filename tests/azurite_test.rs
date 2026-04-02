use az_storage_sharedkey::{
    auth_header::{self, AuthHeader},
    body::Body,
    method::Method,
};
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

fn test_create_container() {
    println!("\nCreate container {CONTAINER} in store-account {TEST_STORE_ACCOUNT}");
    let client = blocking::Client::new();

    let path = format!("/{CONTAINER}");

    let query_pars = [("restype", "container")];
    // first build auth-header witout autorization to be able to extract the
    let mut sr = AuthHeader::new()
        .set_method(Method::Put)
        .set_store_account(
            TEST_STORE_ACCOUNT,
            TEST_STORE_ACCOUNT_KEY_B64,
        )
        .set_dns_suffix(BLOB_SERVICE)
        .set_path(&path)
        .set_query_params(&query_pars)
        .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
//        .insert_header("x-ms-version", "2026-02-06".parse().unwrap())   // this version is not yet supported by Azurite (2026-03-29)
//                                                                           see: https://learn.microsoft.com/en-us/rest/api/storageservices/versioning-for-the-azure-storage-services
        .build();


    let headers = sr.extract_headermap();
    let query_pars = sr.get_query_params();

    let create_container_url_org = format!("{PROTOCOL}://{TEST_STORE_ACCOUNT}.{BLOB_SERVICE}{path}");
    let create_container_url = sr.get_url();

    assert!(create_container_url == create_container_url_org, "mismatch in urls");

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
    let mut sr = AuthHeader::new()
        .set_method(Method::Put)
        .set_store_account(
            &t_a,
            TEST_STORE_ACCOUNT_KEY_B64,
        )
        .set_dns_suffix(BLOB_SERVICE)
        .set_path(&path)
        .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
        .insert_header("x-ms-blob-type", "BlockBlob".parse().unwrap())
        .set_body(Body::Bytes(body_content))
        .build();


    let headers = sr.extract_headermap();

    let create_container_url_org = format!("{PROTOCOL}://{TEST_STORE_ACCOUNT}.{BLOB_SERVICE}{path}");
    let create_container_url = sr.get_url();
    println!("URL: {}", create_container_url);

    assert!(create_container_url == create_container_url_org, "mismatch in urls");

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

    let mut sr = AuthHeader::new()
        .set_method(Method::Get)
        .set_store_account(
            TEST_STORE_ACCOUNT,
            TEST_STORE_ACCOUNT_KEY_B64,
        )
        .set_dns_suffix(BLOB_SERVICE)
        .set_path(&path)
        .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
        .build();


    let headers = sr.extract_headermap();

    let get_container_url_org = format!("{PROTOCOL}://{TEST_STORE_ACCOUNT}.{BLOB_SERVICE}{path}");
    let get_container_url = sr.get_url();

    println!("URL: {}", get_container_url);
    assert!(get_container_url_org == get_container_url, "Mismatch in urls for '{get_container_url}'");

    let res = client.get(get_container_url).headers(headers).send();

    println!("The GET-response: {res:?}");

    let status = res
        .as_ref()
        .expect("Get blob failed with result {res:?}")
        .status();
    assert!(
        status == reqwest::StatusCode::OK,
        "Expected status 200 OK, but observed http-status: {status}"
    );

    let data = res
        .expect("Expected response-success")
        .bytes()
        .expect("Data as bytes in reponse");
    let s = String::from_utf8_lossy(&data);

    println!("Retrieved data: {s}");
}


fn test_exec_get_block_blob() {
    println!(
        "\nGet blob '{BLOB_NAME}' in container '{CONTAINER}' in store-account '{TEST_STORE_ACCOUNT}'."
    );
    let res = AuthHeader::new()
        .set_method(Method::Get)
        .set_store_account(
            TEST_STORE_ACCOUNT,
            TEST_STORE_ACCOUNT_KEY_B64,
        )
        .set_dns_suffix(BLOB_SERVICE)
        .set_path(&format!("/{CONTAINER}/{BLOB_NAME}"))
        .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
        .build()
        .exec_blocking();


    println!("The GET-response: {res:?}");

    let status = res
        .as_ref()
        .expect("Get blob failed with result {res:?}")
        .status();
    assert!(
        status == reqwest::StatusCode::OK,
        "Expected status 200 OK, but observed http-status: {status}"
    );

    let data = res
        .expect("Expected response-success")
        .bytes()
        .expect("Data as bytes in reponse");
    let s = String::from_utf8_lossy(&data);

    println!("Retrieved data: {s}");
}


fn test_delete_block_blob() {
    println!(
        "\nDelete blob '{BLOB_NAME}' in container '{CONTAINER}' in store-account '{TEST_STORE_ACCOUNT}'."
    );
    let client = blocking::Client::new();
    let path = format!("/{CONTAINER}/{BLOB_NAME}");
    

    let mut sr = AuthHeader::new()
        .set_method(Method::Delete)
        .set_store_account(
            TEST_STORE_ACCOUNT,
            TEST_STORE_ACCOUNT_KEY_B64,
        )
        .set_dns_suffix(BLOB_SERVICE)
        .set_path(&path)
        .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
        .build();


    let headers = sr.extract_headermap();

    let delete_container_url = sr.get_url();

    println!("URL: {}", delete_container_url);

    let res = client.delete(delete_container_url).headers(headers).send();

    println!("The DELETE-response: {res:?}");

    let status = res
        .as_ref()
        .expect("Get blob failed with result {res:?}")
        .status();
    assert!(
        status == reqwest::StatusCode::ACCEPTED,
        "Expected status 202 ACCEPTED, but observed http-status: {status}"
    );

}


fn test_delete_container() {
    println!("\nDelete container {CONTAINER} in store-account {TEST_STORE_ACCOUNT}");
    let client = blocking::Client::new();

    let path = format!("/{CONTAINER}");

    let query_pars = [("restype", "container")];
    // first build auth-header witout autorization to be able to extract the
    let mut sr = AuthHeader::new()
        .set_method(Method::Delete)
        .set_store_account(
            TEST_STORE_ACCOUNT,
            TEST_STORE_ACCOUNT_KEY_B64,
        )
        .set_dns_suffix(BLOB_SERVICE)
        .set_path(&path)
        .set_query_params(&query_pars)
        .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
        .build();


    let headers = sr.extract_headermap();
    let query_pars = sr.get_query_params();

    let delete_container_url = sr.get_url();

    println!("URL: {}", delete_container_url);
    let res = client
        .delete(delete_container_url)
        .headers(headers)
        .query(&query_pars)
        .send();

    println!("The DELETE-response: {res:?}");
    let status = res
        .expect("Create container failed with result {res:?}")
        .status();
    assert!(
        status == reqwest::StatusCode::ACCEPTED,
        "Expected status 202 ACCEPTED, but observed http-status: {status}"
    );
}



#[test]
fn run_tests_in_sequence() {
    test_create_container();

    test_create_block_blob();

    test_get_block_blob();

    test_exec_get_block_blob();

    test_delete_block_blob();

    test_delete_container();
}
