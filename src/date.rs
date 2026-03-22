use chrono::{DateTime, TimeZone, Utc};




// Azure storage services follow RFC 1123 for representation of date/time values.
// This is the preferred format for HTTP/1.1 operations, as described in section 3.3 of the HTTP/1.1 Protocol Parameters specification. An example of this format is:
//          Sun, 06 Nov 1994 08:49:37 GMT  


pub fn utc_date_str<T: TimeZone>(dt: &DateTime<T>) -> String {
    dt.to_utc().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

pub fn utc_date_str_now() -> String {
    utc_date_str(&Utc::now())
}
