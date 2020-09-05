use rust_yap::{AppDefBuilder, Group, OptionDef};

fn main() {
    let app_definition = AppDefBuilder::new("abc".into())
        .help(Some("this is an app".into()))
        .add_option_groups(
            vec![(Group::new("basic".into(), Some("basic options".into())), vec![
                OptionDef::new_single_value("verbose".into(), Some('v'), Some("verbose".into()))
                    .help(Some("be verbose".into()))
                    .build()
                    .unwrap()
                    .into()
            ]),
            (Group::new("advanced".into(), Some("advanced options".into())), vec![
                OptionDef::new_multi_value("complex".into(), Some('c'), Some("complex".into()))
                    .help(Some("complex option".into()))
                    .build()
                    .unwrap()
                    .into(),
                OptionDef::new_multi_value("color".into(), None, Some("color".into()))
                    .min_occurences(3)
                    .max_occurences(Some(3))
                    .help(Some("color RGB values".into()))
                    .build()
                    .unwrap()
                    .into()
            ])]
        ).unwrap()
         .add_option(
            OptionDef::new_single_value("secret".into(), Some('s'), Some("secret".into()))
                .help(Some("be secret".into()))
                .build()
                .unwrap()
                .into()
        )
        .build()
        .unwrap();
    app_definition.print_usage();
}
