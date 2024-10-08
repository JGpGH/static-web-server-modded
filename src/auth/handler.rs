//! Handle authorization
//!
use std::path::Path;

use http::{Method, Uri};
use hyper::{ Body, Request, Response, StatusCode};

use crate::{error_page, handler::RequestHandlerOpts, Error};

use super::auth_client::AuthClient;

struct _ResponseBuilder<'a, 'b> {
    uri: &'b Uri,
    method: &'b Method,
    page404: &'a Path,
    page50x: &'a Path,
}

impl<'a, 'b> _ResponseBuilder<'a, 'b> {
    pub fn _from<T>(opts: &'a RequestHandlerOpts, request: &'b Request<T>) -> Self {
        Self {
            uri: request.uri(),
            method: request.method(),
            page404: &opts.page404,
            page50x: &opts.page50x,
        }
    }

    pub fn _my_bad(&self, reason: &str) -> Response<Body> {
        tracing::error!(reason);
        error_page::error_response(
            &self.uri,
            &self.method,
            &StatusCode::INTERNAL_SERVER_ERROR,
            self.page404,
            self.page50x,
        ).unwrap()
    }

    pub fn _unauthorized(&self) -> Response<Body> {
        error_page::error_response(
            &self.uri,
            &self.method,
            &StatusCode::UNAUTHORIZED,
            self.page404,
            self.page50x,
        ).unwrap()
    }
}

/// Handles `Basic` HTTP Authorization Schema
pub(crate) fn pre_process<T>(
    _opts: &RequestHandlerOpts,
    _auth_client: &AuthClient,
    _req: &Request<T>,
) -> Option<Result<Response<Body>, Error>> {
    return None
}