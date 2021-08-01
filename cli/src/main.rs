// Copyright 2021 Remi Bernotavicius

use std::path::PathBuf;
use std::{io, net};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    #[structopt(long)]
    do_not_open_browser: bool,

    #[structopt(long, default_value = "localhost")]
    host: String,
}

fn main() -> io::Result<()> {
    simple_logger::SimpleLogger::new().init().unwrap();

    let opt = Opt::from_args();
    let tree = vdu::build_tree_from_path(&opt.path)?;

    let socket = net::TcpListener::bind(&format!("{}:0", opt.host))?;
    let port = socket.local_addr()?.port();

    let url = format!("http://{}:{}/", opt.host, port);
    log::info!("visit {} to see results", url);

    if !opt.do_not_open_browser {
        log::info!("opening browser");
        webbrowser::open(&url).unwrap();
    }

    vdu::run_server(tree, socket)
}
