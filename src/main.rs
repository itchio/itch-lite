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
