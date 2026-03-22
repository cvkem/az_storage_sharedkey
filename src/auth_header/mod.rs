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

    const TEST_STORE_ACCOUNT_KEY_B64: &str =
        "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";

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
            ]);


        let to_sign = auth_header.get_string_to_sign();
        // FOR Debugging only
        println!("to-sign = {}", to_sign);

        // comparison withou field 'x-ms-date'
        assert!(to_sign == "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-version:2015-02-21\n/myaccount/mycontainer\ncomp:metadata\nrestype:container\ntimeout:20",
//        "GET\n\n\n\n\n\n\n\n\n\n\n\nx-ms-date:Fri, 26 Jun 2015 23:39:12 GMT\nx-ms-version:2015-02-21\n/myaccount/mycontainer\ncomp:metadata\nrestype:container\ntimeout:20"
          "The string to sign is '{to_sign}'"
    )

    }

}