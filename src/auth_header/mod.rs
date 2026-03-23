mod auth_header;
mod hmac_sha256;

pub const GET: &str = "GET";
pub const PUT: &str = "PUT";
pub const POST: &str = "POST";

pub use auth_header::AuthHeader;


#[cfg(test)]
mod test {

    use crate::{
    auth_header::{self, AuthHeader},
    date::{utc_date_str, utc_date_str_now},
    };

    use chrono::{TimeZone, Utc};

    const TEST_STORE_ACCOUNT: &str = "devstoreaccount1";
    const TEST_STORE_ACCOUNT_KEY_B64: &str =
        "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";

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


    #[test]
    fn test_string_to_sign() {
        // Building up next request
        // GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Fri, 26 Jun 2015 23:39:12 GMT\nx-ms-version:2015-02-21\n/myaccount/mycontainer\ncomp:metadata\nrestype:container\ntimeout:20
        // Authorization: SharedKey myaccount:ctzMq410TV3wS7upTBcunJTDLEJwMAZuFPfr0mrrA08=

        // the default storage account
        // DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;QueueEndpoint=http://127.0.0.1:10001/devstoreaccount1;TableEndpoint=http://127.0.0.1:10002/devstoreaccount1;DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;QueueEndpoint=http://127.0.0.1:10001/devstoreaccount1;TableEndpoint=http://127.0.0.1:10002/devstoreaccount1;DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;QueueEndpoint=http://127.0.0.1:10001/devstoreaccount1;TableEndpoint=http://127.0.0.1:10002/devstoreaccount1;

        let dt = Utc.with_ymd_and_hms(2015, 6, 26, 23, 39, 12).unwrap();
        println!("The date = {dt:?}");

        let auth_header = AuthHeader::new()
            .set_method(auth_header::GET)
            .set_store_account(
                "myaccount",
                TEST_STORE_ACCOUNT_KEY_B64,
            )
            .set_path("/mycontainer".to_owned())
            .set_datetime(dt)
            .insert_header("x-ms-version", "2015-02-21".parse().unwrap())
            .set_query_params(&[
                ("comp", "metadata"),
                ("restype", "container"),
                ("timeout", "20"),
            ])
            .build();


        let to_sign = auth_header.get_unsigned_authorization();
        // FOR Debugging only
        println!("to-sign = {}", to_sign);

        // comparison withou field 'x-ms-date'
        assert!(check_to_sign_without_xmsdate(to_sign, "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Fri, 26 Jun 2015 23:39:12 GMT\nx-ms-version:2015-02-21\n/myaccount/mycontainer\ncomp:metadata\nrestype:container\ntimeout:20"),
          "The string to sign is '{to_sign}'"
    )

    }


    #[test]
    fn test_create_container_sign_string() {
        println!("\nCheck sign-string for Create container {CONTAINER} in store-account {TEST_STORE_ACCOUNT}");

        let path = format!("/{CONTAINER}");

        let query_pars = [("restype", "container")];
        // first build auth-header witout autorization to be able to extract the
        let sr = AuthHeader::new()
            .set_method(auth_header::PUT)
            .set_store_account(
                TEST_STORE_ACCOUNT,
                TEST_STORE_ACCOUNT_KEY_B64,
            )
            .set_path(path.to_owned())
            .set_query_params(&query_pars)
            .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
            .build();

        let to_sign = sr.get_unsigned_authorization();
        assert!(check_to_sign_without_xmsdate(
            &to_sign,
            "PUT\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Sat, 14 Mar 2026 15:11:55 GMT\nx-ms-version:2019-12-12\n/devstoreaccount1/container\nrestype:container"
        ), "Failure on the sign-string for Create-container '{to_sign}'.");
    }


    #[test]
    fn test_create_block_blob_sign_string() {
        let body_content = BLOB_CONTENT.as_bytes();
        println!(
            "\nCheck sign-string for Create blob '{BLOB_NAME}' in container '{CONTAINER}' in store-account '{TEST_STORE_ACCOUNT}'."
        );

        let path = format!("/{CONTAINER}/{BLOB_NAME}");

        // I can also pass a reference to a lokal string as this is guaranteed to exit long enough
        let t_a = TEST_STORE_ACCOUNT.to_owned();
        // first build auth-header witout autorization to be able to extract the
        let sr = AuthHeader::new()
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
            .set_query_params(&[])
            .build();

        let to_sign = sr.get_unsigned_authorization();
        assert!(check_to_sign_without_xmsdate(
            &to_sign,
           "PUT\n\n\n12\n\n\n\n\n\n\n\n\nx-ms-blob-type:BlockBlob\nx-ms-date:Sat, 14 Mar 2026 15:11:55 GMT\nx-ms-version:2019-12-12\n/devstoreaccount1/container/blob_name"
        ), "The string to sign does not match. Received '{to_sign}'"
    );
    }


    #[test]
    fn test_get_block_blob_sign_string() {
        println!(
            "\nGet blob '{BLOB_NAME}' in container '{CONTAINER}' in store-account '{TEST_STORE_ACCOUNT}'."
        );

        let path = format!("/{CONTAINER}/{BLOB_NAME}");

        let sr = AuthHeader::new()
            .set_method(auth_header::GET)
            .set_store_account(
                TEST_STORE_ACCOUNT,
                TEST_STORE_ACCOUNT_KEY_B64,
            )
            .set_path(path.to_owned())
            .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
            .set_query_params(&[])
            .build();

        let to_sign = sr.get_unsigned_authorization();
        assert!(check_to_sign_without_xmsdate(
            &to_sign,
            "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Sat, 14 Mar 2026 15:11:55 GMT\nx-ms-version:2019-12-12\n/devstoreaccount1/container/blob_name"
        ), "Sign-string for get-blob does not match: '{to_sign}'.");
    }

}