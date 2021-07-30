// Copyright 2021 Remi Bernotavicius

use std::path::PathBuf;
use std::{io, net};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    path: PathBuf,
}

fn main() -> io::Result<()> {
    simple_logger::SimpleLogger::new().init().unwrap();

    let opt = Opt::from_args();
    let tree = vdu::build_tree_from_path(&opt.path)?;

    let socket = net::TcpListener::bind("127.0.0.1:0")?;
    let port = socket.local_addr()?.port();

    let url = format!("http://127.0.0.1:{}/", port);
    log::info!("opening {}", url);

    webbrowser::open(&url).unwrap();

    vdu::run_server(tree, socket)
}
