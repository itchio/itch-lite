struct Handler {}

impl Drop for Handler {
    fn drop(&mut self) {
        println!("Exiting");
        tether::exit();
    }
}

impl tether::Handler for Handler {
    fn handle_rpc(&mut self, _window: tether::Window, msg: &str) {
        println!("[rpc] received {}", msg);
    }

    fn handle_net(&mut self, req: tether::NetRequest) -> Result<(), Box<dyn std::error::Error>> {
        println!("[net] requesting {}", req.uri());

        if req.uri().find("127.0.0.1").is_some() {
            println!("intercepting request!");

            let s = "hello from itch-lite";
            req.respond(tether::NetResponse {
                status_code: 200,
                content: s.as_bytes(),
            });
        } else {
            println!("letting request through");
        }

        Ok(())
    }
}

fn main() {
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
    win.load(include_str!("./resources/index.html"));
}
