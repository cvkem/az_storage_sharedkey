use std::fmt::{Display, Formatter, Result};

// GET: The GET method requests a representation of the specified resource. Requests using GET should only retrieve data and should not contain a request content.
// HEAD: The HEAD method asks for a response identical to a GET request, but without a response body.
// POST: The POST method submits an entity to the specified resource, often causing a change in state or side effects on the server.
// PUT: The PUT method replaces all current representations of the target resource with the request content.
// DELETE: The DELETE method deletes the specified resource.
// CONNECT: The CONNECT method establishes a tunnel to the server identified by the target resource.
// OPTIONS: The OPTIONS method describes the communication options for the target resource.
// TRACE: The TRACE method performs a message loop-back test along the path to the target resource.
// PATCH: The PATCH method applies partial modifications to a resource.

#[derive(Clone, Copy)]
pub enum Method {
    None, // indicates no Method was selected yet.
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            Method::None => "--",
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Connect => "CONNECT",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
            Method::Patch => "PATCH",
        })
    }
}
