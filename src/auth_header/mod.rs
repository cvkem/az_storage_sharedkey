mod auth_header;
mod hmac_sha256;

// pub const GET: &str = "GET";
// pub const PUT: &str = "PUT";
// pub const POST: &str = "POST";
// pub const DELETE: &str = "DELETE";



pub use auth_header::AuthHeader;
pub const MSDATE_KEY: &str = "x-ms-date";



#[cfg(test)]
mod test {

    use crate::{
        auth_header::{self, AuthHeader},
        body::Body,
        method::Method,
    };

    use chrono::{TimeZone, Utc};
    use reqwest::header::HeaderName;


    const TEST_STORE_ACCOUNT: &str = "devstoreaccount1";
    const TEST_STORE_ACCOUNT_KEY_B64: &str =
        "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";

    const CONTAINER: &str = "container";
    const BLOB_NAME: &str = "blob_name";
    const BLOB_CONTENT: &str = "Hello world!";

    const BLOB_SERVICE: &str = "blob.local";



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
    #[should_panic(expected = "Use the method 'self.set_date(...)' to add a date-headers.")]
    fn test_no_x_ms_date_header() {
        let _auth_header = AuthHeader::new()
            .insert_header(auth_header::MSDATE_KEY, "2015-02-21".parse().unwrap());
    }

    #[test]
    #[should_panic]
    fn test_no_content_length_header() {
        let _auth_header = AuthHeader::new()
            .insert_header(reqwest::header::CONTENT_LENGTH.as_str(), "123".parse().unwrap());
    }

    #[test]
    fn test_add_body_and_extract_it() {
        let body = "abc";
        let sr = AuthHeader::new()
            .set_store_account(
                "myaccount",
                TEST_STORE_ACCOUNT_KEY_B64,
            )
            .set_dns_suffix(BLOB_SERVICE)
//            .set_path("/mycontainer".to_owned())
//            .insert_header("x-ms-version", "2015-02-21".parse().unwrap())
            .set_body(Body::Text(body))
            .build();

        let body_ref = sr.body_as_bytes();
        let body_ref2 = sr.body_as_bytes();
        assert!(str::from_utf8(body_ref.unwrap()).unwrap() == body, "Extracted body should match the original body passed as input");
        assert!(str::from_utf8(body_ref2.unwrap()).unwrap() == body, "Extracted body should match the original body passed as input");
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

        let storage_request = AuthHeader::new()
            .set_method(Method::Get)
            .set_store_account(
                "myaccount",
                TEST_STORE_ACCOUNT_KEY_B64,
            )
            .set_dns_suffix(BLOB_SERVICE)
            .set_path("/mycontainer")
            .set_datetime(dt)
            .insert_header("x-ms-version", "2015-02-21".parse().unwrap())
            .set_query_params(&[
                ("comp", "metadata"),
                ("restype", "container"),
                ("timeout", "20"),
            ])
            .build();


        let to_sign = storage_request.get_unsigned_authorization();
        // FOR Debugging only
        println!("to-sign = {}", to_sign);

        // comparison withou field 'x-ms-date'
        assert!(check_to_sign_without_xmsdate(to_sign, "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Fri, 26 Jun 2015 23:39:12 GMT\nx-ms-version:2015-02-21\n/myaccount/mycontainer\ncomp:metadata\nrestype:container\ntimeout:20"),
          "The string to sign is '{to_sign}'"
    )

    }

        #[test]
    fn test_string_to_sign_full() {
        // Building up next request
        // GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Fri, 26 Jun 2015 23:39:12 GMT\nx-ms-version:2015-02-21\n/myaccount/mycontainer\ncomp:metadata\nrestype:container\ntimeout:20
        // Authorization: SharedKey myaccount:ctzMq410TV3wS7upTBcunJTDLEJwMAZuFPfr0mrrA08=

        // the default storage account
        // DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;QueueEndpoint=http://127.0.0.1:10001/devstoreaccount1;TableEndpoint=http://127.0.0.1:10002/devstoreaccount1;DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;QueueEndpoint=http://127.0.0.1:10001/devstoreaccount1;TableEndpoint=http://127.0.0.1:10002/devstoreaccount1;DefaultEndpointsProtocol=http;AccountName=devstoreaccount1;AccountKey=Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==;BlobEndpoint=http://127.0.0.1:10000/devstoreaccount1;QueueEndpoint=http://127.0.0.1:10001/devstoreaccount1;TableEndpoint=http://127.0.0.1:10002/devstoreaccount1;

        let dt = Utc.with_ymd_and_hms(2015, 6, 26, 23, 39, 12).unwrap();
        println!("The date = {dt:?}");

        let auth_header = AuthHeader::new()
            .set_method(Method::Get)
            .set_store_account(
                "myaccount",
                TEST_STORE_ACCOUNT_KEY_B64,
            )
            .set_dns_suffix(BLOB_SERVICE)
            .set_path("/mycontainer")
            .set_datetime(dt)
            .set_content_length_without_body(123)
            .insert_header("x-ms-version", "2015-02-21".parse().unwrap())
            .insert_header(reqwest::header::CONTENT_ENCODING, "gzip".parse().unwrap())
            .insert_header(reqwest::header::CONTENT_LANGUAGE, "nl-NL".parse().unwrap())
            .insert_header(&HeaderName::from_static("content_md5"), "1a2b3c".parse().unwrap())
            .insert_header(reqwest::header::CONTENT_TYPE, "application/octet-stream".parse().unwrap())
            .insert_header(reqwest::header::DATE, "Tue, 29 Oct 2024 16:56:32 GMT".parse().unwrap())
            .insert_header(reqwest::header::IF_MODIFIED_SINCE, "Wed, 21 Oct 2015 07:28:00 GMT".parse().unwrap())       
            .insert_header(reqwest::header::IF_MATCH, "\"67ab43\"".parse().unwrap())       
            .insert_header(reqwest::header::IF_NONE_MATCH, "\"abc\"".parse().unwrap())       
            .insert_header(reqwest::header::IF_UNMODIFIED_SINCE, "Wed, 14 Oct 2015 08:29:00 GMT".parse().unwrap())       
            .insert_header(reqwest::header::RANGE, "bytes=500-999".parse().unwrap())       
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
        assert!(check_to_sign_without_xmsdate(to_sign, "GET\ngzip\nnl-NL\n123\n1a2b3c\napplication/octet-stream\nTue, 29 Oct 2024 16:56:32 GMT\nWed, 21 Oct 2015 07:28:00 GMT\n\"67ab43\"\n\"abc\"\nWed, 14 Oct 2015 08:29:00 GMT\nbytes=500-999\nx-ms-date:Fri, 26 Jun 2015 23:39:12 GMT\nx-ms-version:2015-02-21\n/myaccount/mycontainer\ncomp:metadata\nrestype:container\ntimeout:20"),
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
            .set_method(Method::Put)
            .set_store_account(
                TEST_STORE_ACCOUNT,
                TEST_STORE_ACCOUNT_KEY_B64,
            )
            .set_dns_suffix(BLOB_SERVICE)
            .set_path(&path)
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
        let body_content = BLOB_CONTENT;
        println!(
            "\nCheck sign-string for Create blob '{BLOB_NAME}' in container '{CONTAINER}' in store-account '{TEST_STORE_ACCOUNT}'."
        );

        let path = format!("/{CONTAINER}/{BLOB_NAME}");

        // I can also pass a reference to a lokal string as this is guaranteed to exit long enough
        let t_a = TEST_STORE_ACCOUNT.to_owned();
        // first build auth-header witout autorization to be able to extract the
        let sr = AuthHeader::new()
            .set_method(Method::Put)
            .set_store_account(
                &t_a,
                TEST_STORE_ACCOUNT_KEY_B64,
            )
            .set_dns_suffix(BLOB_SERVICE)
            .set_path(&path)
            .insert_header("x-ms-version", "2019-12-12".parse().unwrap())
            .insert_header("x-ms-blob-type", "BlockBlob".parse().unwrap())
            .set_body(Body::Text(body_content))
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
            .set_dns_suffix(BLOB_SERVICE)
            .set_method(Method::Get)
            .set_store_account(
                TEST_STORE_ACCOUNT,
                TEST_STORE_ACCOUNT_KEY_B64,
            )
            .set_path(&path)
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