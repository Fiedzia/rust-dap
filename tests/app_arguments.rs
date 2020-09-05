use rust_yap::{AppDefBuilder, ArgumentDef};


#[test]
fn test_app_with_single_required_argument() {
    let app_definition = AppDefBuilder::new("abc".to_string())
        .add_argument(
                ArgumentDef::new_single_value("color".to_string())
                    .required(true)
                    .build()
                    .unwrap()
                    .into()
        )
        .build()
        .unwrap();

    let app = app_definition.parse_args(&[]);
    assert_eq!(app.is_ok(), false);

    let app = app_definition.parse_args(&["red"]).unwrap();
    assert_eq!(app.single_value_arguments.get("color"), Some(&"red".to_string()));
}

#[test]
fn test_app_with_three_single_required_arguments() {
    let app_definition = AppDefBuilder::new("food_delivery".to_string())
        .add_argument(
                ArgumentDef::new_single_value("main".to_string())
                    .required(true)
                    .build()
                    .unwrap()
                    .into()
        )
        .add_argument(
                ArgumentDef::new_single_value("drink".to_string())
                    .required(true)
                    .build()
                    .unwrap()
                    .into()
        )
        .add_argument(
                ArgumentDef::new_single_value("side".to_string())
                    .required(true)
                    .build()
                    .unwrap()
                    .into()
        )
        .build()
        .unwrap();

    let app = app_definition.parse_args(&[]);
    assert_eq!(app.is_ok(), false);

    let app = app_definition.parse_args(&["pizza"]);
    assert_eq!(app.is_ok(), false);

    let app = app_definition.parse_args(&["pizza", "orange juice"]);
    assert_eq!(app.is_ok(), false);


    let app = app_definition.parse_args(&["pizza", "orange juice", "coleslaw"]).unwrap();
    assert_eq!(app.single_value_arguments.get("main"), Some(&"pizza".to_string()));
    assert_eq!(app.single_value_arguments.get("drink"), Some(&"orange juice".to_string()));
    assert_eq!(app.single_value_arguments.get("side"), Some(&"coleslaw".to_string()));

    let app = app_definition.parse_args(&["pizza", "orange juice", "coleslaw", "salt"]);
    assert_eq!(app.is_ok(), false);
}

