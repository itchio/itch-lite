use web_view::WebViewBuilder;

fn main() {
    let app = WebViewBuilder::new()
        .title("itch lite")
        .content(web_view::Content::Html(include_str!(
            "./resources/index.html"
        )))
        .size(1280, 720)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .build()
        .unwrap();
    app.run().unwrap();
}
