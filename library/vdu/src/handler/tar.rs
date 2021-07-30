use http_io::error::{Error, Result};
use http_io::protocol::{HttpBody, HttpResponse, HttpStatus};
use http_io::server::HttpRequestHandler;
use std::collections::HashMap;
use std::io;
use std::path::Path;

fn mime_for_path(path: &str) -> &'static str {
    if let Some(ext) = Path::new(path).extension() {
        match &ext.to_str().unwrap().to_lowercase()[..] {
            "wasm" => return "application/wasm",
            "js" => return "text/javascript",
            "html" => return "text/html",
            _ => (),
        }
    }

    log::warn!("mime for '{}' unknown", path);
    "application/octet-stream"
}

pub struct TarHandler {
    map: HashMap<String, &'static [u8]>,
}

impl TarHandler {
    pub fn from_memory(bytes: &'static [u8]) -> Self {
        let mut map = HashMap::new();
        let mut ar = tar::Archive::new(bytes);
        for entry in ar.entries().unwrap() {
            let entry = entry.unwrap();
            let header = entry.header();

            let path = header.path().unwrap().to_str().unwrap().into();
            let start = entry.raw_file_position() as usize;
            let end = start + header.size().unwrap() as usize;
            map.insert(path, &bytes[start..end]);
        }
        Self { map }
    }

    fn get_file(&self, path: &str) -> Option<HttpResponse<Box<dyn io::Read>>> {
        let mut path = format!(".{}", path);

        if path == "./" {
            path = "./index.html".into();
        }

        self.map.get(&path[..]).map(|&b| {
            let mut response = HttpResponse::new(HttpStatus::OK, Box::new(b) as Box<dyn io::Read>);
            response.add_header("Content-Type", mime_for_path(&path));
            response
        })
    }
}

impl<I: io::Read> HttpRequestHandler<I> for TarHandler {
    type Error = Error;
    fn get(&mut self, uri: String) -> Result<HttpResponse<Box<dyn io::Read>>> {
        if let Some(resp) = self.get_file(&uri[..]) {
            Ok(resp)
        } else {
            Ok(HttpResponse::new(
                HttpStatus::NotFound,
                Box::new(io::empty()),
            ))
        }
    }

    fn put(
        &mut self,
        _uri: String,
        _stream: HttpBody<&mut I>,
    ) -> Result<HttpResponse<Box<dyn io::Read>>> {
        Ok(HttpResponse::new(
            HttpStatus::NotFound,
            Box::new(io::empty()),
        ))
    }
}
