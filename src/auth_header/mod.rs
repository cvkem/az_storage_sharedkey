mod auth_header;
mod hmac_sha256;


pub const GET: &str = "GET";
pub const PUT: &str = "PUT";
pub const POST: &str = "POST";

pub use auth_header::AuthHeader;