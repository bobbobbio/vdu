// copyright Remi Bernotavicius 2021

use http_io::server::{HttpServer, Listen};
use std::io::Result;

use vdu_path_tree::PathTree;
pub use walk::build_tree_from_path;

mod handler;
mod walk;

pub fn run_server<S: Listen>(tree: PathTree, connection_stream: S) -> Result<()> {
    let mut server = HttpServer::new(connection_stream, handler::VduHandler::new(tree));
    loop {
        server.serve_one()?
    }
}
