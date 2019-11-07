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
        info!("[net] requesting {}", req.uri());

        if req.uri().find("127.0.0.1").is_some() {
            info!("intercepting request!");

            let s = include_str!("./resources/index.html");
            req.respond(tether::NetResponse {
                status_code: 200,
                content: s.as_bytes(),
            });
        } else {
            info!("letting request through");
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
    win.navigate("http://127.0.0.1/index.html");
}
