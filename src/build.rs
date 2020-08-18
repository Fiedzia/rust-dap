extern crate lalrpop;

fn main() {
    lalrpop::Configuration::new().process_current_dir().unwrap();
    //lalrpop::.process_root().unwrap();
}
