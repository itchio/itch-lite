use log::*;

struct Handler {}

impl Drop for Handler {
    fn drop(&mut self) {
        info!("Exiting");
        tether::exit();
    }
}

impl tether::Handler for Handler {
    fn handle_rpc(&mut self, _window: tether::Window, msg: &str) {
        info!("[rpc] received {}", msg);
    }

    fn handle_net(&mut self, req: tether::NetRequest) -> Result<(), Box<dyn std::error::Error>> {
        let url = req.url();
        info!("[net] requesting {:?}", url);

        match url.host_str() {
            Some("itch-lite") => {
                info!("intercepting request!");

                let path = url.path();
                let path = percent_encoding::percent_decode(path.as_bytes()).decode_utf8()?;
                let path = path.trim_start_matches("/");
                info!("path = {:?}", path);

                let file_path = std::path::PathBuf::from("src").join("resources").join(path);
                info!("file_path = {:?}", file_path);

                match std::fs::read(file_path) {
                    Ok(f) => {
                        req.respond(tether::NetResponse {
                            status_code: 200,
                            content: &f[..],
                        });
                        return Ok(());
                    }
                    Err(_) => {
                        req.respond(tether::NetResponse {
                            status_code: 404,
                            content: "not found".as_bytes(),
                        });
                        return Ok(());
                    }
                }
            }
            _ => {
                info!("letting request through");
            }
        }

        Ok(())
    }
}

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter(None, log::LevelFilter::Info).init();

    info!("Starting up!");

    unsafe {
        tether::start(start);
    }
}

fn start() {
    let win = tether::Window::new(tether::Options {
        debug: true,
        initial_width: 1280,
        initial_height: 720,
        handler: Some(Box::new(Handler {})),
        ..Default::default()
    });

    win.title("itch lite");
    // win.navigate("http://itch-lite/index.html");
    win.load(include_str!("./resources/index.html"));
}
