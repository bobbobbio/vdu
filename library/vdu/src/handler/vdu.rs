// copyright 2021 Remi Bernotavicius

use super::TarHandler;
use http_io::error::{Error, Result};
use http_io::protocol::{HttpBody, HttpResponse, HttpStatus};
use http_io::server::HttpRequestHandler;
use std::io;

use vdu_path_tree::PathTree;

const WEB_TAR: &'static [u8] = include_bytes!("../../web.tar");

pub struct VduHandler {
    tar: TarHandler,
    tree: PathTree,
}

impl VduHandler {
    pub fn new(tree: PathTree) -> Self {
        Self {
            tar: TarHandler::from_memory(WEB_TAR),
            tree,
        }
    }

    fn get_tree(&self) -> HttpResponse<Box<dyn io::Read>> {
        let data = bincode::serialize(&self.tree).unwrap();
        let body: Box<dyn io::Read> = Box::new(io::Cursor::new(data));
        let mut response = HttpResponse::new(HttpStatus::OK, body);
        response.add_header("Content-Type", "application/octet-stream");
        response
    }
}

impl<I: io::Read> HttpRequestHandler<I> for VduHandler {
    type Error = Error;
    fn get(&mut self, uri: String) -> Result<HttpResponse<Box<dyn io::Read>>> {
        if uri == "/tree" {
            Ok(self.get_tree())
        } else {
            <TarHandler as HttpRequestHandler<I>>::get(&mut self.tar, uri)
        }
    }

    fn put(
        &mut self,
        uri: String,
        stream: HttpBody<&mut I>,
    ) -> Result<HttpResponse<Box<dyn io::Read>>> {
        <TarHandler as HttpRequestHandler<I>>::put(&mut self.tar, uri, stream)
    }
}
