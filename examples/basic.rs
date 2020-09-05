use rust_yap::AppDefBuilder;

fn main() {
    let app_definition = AppDefBuilder::new("abc".to_string())
        .help(Some("this is an app".to_string()))
        .build()
        .unwrap();
    app_definition.print_usage();

}


