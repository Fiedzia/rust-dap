//
//TODO:
//     detect ambiguity
//     use colors
//     align help output for flags/options
//     handle --help and help command
//     display ungrouped options before grouped ones
//     display nice error messages
//     validation
//     support argument allowed values, default values
//     handle positional separator

use std::convert::From;

#[macro_use]
extern crate derivative;
use indexmap::{IndexMap, IndexSet, map::Entry};

mod parser;

#[macro_use]
extern crate derive_builder;


type ValueValidator = fn(&str) -> Result<(), String>;
type MultiValueValidator = fn(&Vec<String>) -> Result<(), String>;
type SubCommandValidator = fn(&SubCommandDef) -> Result<(), String>;
type AppValidator = fn(&App) -> Result<(), String>;


#[derive(Clone, Default)]
pub struct ArgumentDefs(pub Vec<ArgumentDef>);

impl ArgumentDefs {

    pub fn validate(&self, single_value_arguments: &IndexMap<String, String>, multi_value_arguments: &IndexMap<String, Vec<String>>) -> Result<(), String> {
        for argument_def in &self.0 {
            argument_def.validate(&single_value_arguments, &multi_value_arguments)?
        }
        Ok(())
    }
}


#[derive(Clone, Default)]
pub struct FlagDefs(pub Vec<FlagDef>);

impl FlagDefs {

    pub fn add_flag(&mut self, flag_def: FlagDef) -> &mut Self {
        self.0.push(flag_def);
        self
    }

    pub fn by_short(&self, ch: &char) -> Option<&FlagDef> {
        for fd in &self.0 {
            match fd {
                FlagDef::BooleanFlagDef(ref bfd) => if let Some(v) = bfd.short {
                    if  v == *ch {
                        return Some(&fd);
                    }
                },
                FlagDef::CountedFlagDef(ref cfd) => if let Some(v) = cfd.short {
                    if  v == *ch {
                        return Some(&fd);
                    }
                },

            }
        }
        None
    }

    pub fn by_long(&self, param: &str) -> Option<&FlagDef> {
        for fd in &self.0 {
            match fd {
                FlagDef::BooleanFlagDef(ref bfd) => if let Some(v) = &bfd.long {
                    if  v == param {
                        return Some(&fd);
                    }
                },
                FlagDef::CountedFlagDef(ref cfd) => if let Some(v) = &cfd.long {
                    if  v == param {
                        return Some(&fd);
                    }
                },

            }
        }
        None
    }

    pub fn validate(&self, boolean_flags: &IndexMap<String, bool>, counted_flags: &IndexMap<String, u64>) -> Result<(), String> {
        for flag_def in &self.0 {
            flag_def.validate(&boolean_flags, &counted_flags)?
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct OptionDefs(pub Vec<OptionDef>);

impl OptionDefs {

    pub fn by_name(&self, name: &str) -> Option<&OptionDef> {
        for item in &self.0 {
            match item {
                OptionDef::SingleValue(svod) => {
                    if svod.name == name {
                        return Some(&item);
                    }
                },
                OptionDef::MultiValue(mvod) => {
                    if mvod.name == name {
                        return Some(&item);
                    }
                },
            }
        }
        None
    }

    pub fn by_short(&self, ch: &char) -> Option<&OptionDef> {
        for od in &self.0 {
            match od {
                OptionDef::SingleValue(ref svod) => if let Some(v) = svod.short {
                    if  v == *ch {
                        return Some(&od);
                    }
                },
                OptionDef::MultiValue(ref mvod) => if let Some(v) = mvod.short {
                    if  v == *ch {
                        return Some(&od);
                    }
                },

            }
        }
        None
    }

    pub fn by_long(&self, param: &str) -> Option<&OptionDef> {
        for od in &self.0 {
            match od {
                OptionDef::SingleValue(ref svod) => if let Some(v) = &svod.long {
                    if  v == param {
                        return Some(&od);
                    }
                },
                OptionDef::MultiValue(ref mvod) => if let Some(v) = &mvod.long {
                    if  v == param {
                        return Some(&od);
                    }
                },

            }
        }
        None
    }


    pub fn validate(&self, single_value_options: &IndexMap<String, String>, multi_value_options: &IndexMap<String, Vec<String>>) -> Result<(), String> {
        for option_def in &self.0 {
            option_def.validate(&single_value_options, &multi_value_options)?
        }
        Ok(())
    }
}


#[derive(Builder, Clone)]
pub struct Group {
    #[builder(default = "None")]
    items: Option<IndexSet<String>>,
    name: String,
    #[builder(default = "None")]
    help: Option<String>,
}

impl Group {
    pub fn new(name: String, help: Option<String>) -> Group {
        Group {
            name,
            help,
            items: None
        }
    }

    pub fn add_item(&mut self, item: &str) {
        match &mut self.items {
            Some(is) => { is.insert(item.to_string()); },
            None => {
                let mut is = IndexSet::new();
                is.insert(item.to_string());
                self.items = Some(is);
            }
        }
    }
}


#[derive(Builder, Clone)]
pub struct BooleanFlagDef {
    name: String,
    #[builder(default = "None")]
    short: Option<char>,
    #[builder(default = "None")]
    long: Option<String>,
    #[builder(default = "None")]
    help: Option<String>,
    #[builder(default = "false")]
    required: bool,
}

impl BooleanFlagDef {
    pub fn to_flag_def(self) -> FlagDef {
        FlagDef::BooleanFlagDef(self)
    }

    pub fn validate(&self, boolean_flags: &IndexMap<String, bool>) ->  Result<(), String> {
        if self.required && !boolean_flags.contains_key(&self.name) {
            return Err(format!("flag {} is required", self.name))
        }
        Ok(())
    }
}

impl BooleanFlagDefBuilder {
    pub fn new(name: String, short: Option<char>, long: Option<String>) -> BooleanFlagDefBuilder {
        let mut bfdb = BooleanFlagDefBuilder::default();
        bfdb.name(name)
            .short(short)
            .long(long);
        bfdb
    }
}


#[derive(Builder, Clone)]
#[builder(build_fn(validate = "Self::validate_def"))]
pub struct CountedFlagDef {
    name: String,
    #[builder(default = "None")]
    short: Option<char>,
    #[builder(default = "None")]
    long: Option<String>,
    #[builder(default = "None")]
    help: Option<String>,
    #[builder(default = "0")]
    min_occurences: u64,
    #[builder(default = "None")]
    max_occurences: Option<u64>,
}

impl CountedFlagDef {
    pub fn to_flag_def(self) -> FlagDef {
        FlagDef::CountedFlagDef(self)
    }

    pub fn validate(&self, counted_flags: &IndexMap<String, u64>) ->  Result<(), String> {
        let cnt = match counted_flags.get(&self.name) {
            Some(v) => *v,
            None => 0,
        };
        if cnt < self.min_occurences {
            return Err(format!("flag {} must appear at least {} time(s), it appeared {} time(s)", self.name, self.min_occurences, cnt));
        
        };
        if let Some(mx) = self.max_occurences {
            if cnt > mx {
                return Err(format!("flag {} may appear at most {} time(s), it appeared {} time(s)", self.name, self.min_occurences, cnt));
            }
        }
        Ok(())

    }


}

impl CountedFlagDefBuilder {

    pub fn new(name: String, short: Option<char>, long: Option<String>) -> CountedFlagDefBuilder {
        let mut cfdb = CountedFlagDefBuilder::default();
        cfdb.name(name)
            .short(short)
            .long(long);
        cfdb
    }

    fn validate_def(&self) -> Result<(), String> {
        if let (None, None) = (&self.short, &self.long) {
            return Err(format!("{:?}: either short or long name must be provided", self.name))
        };

        if self.min_occurences == Some(0) && self.max_occurences == Some(Some(0)) {
            return Err(format!("{:?}: min/max_occurences cannot be both set to 0", self.name))
        }

        if let (Some(v1), Some(Some(v2))) = (self.min_occurences, self.max_occurences) {
            if v1 > v2 {
                return Err(format!("{:?}: min_occurences must not exceed max_occurences", self.name))
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub enum FlagDef {
    BooleanFlagDef(BooleanFlagDef),
    CountedFlagDef(CountedFlagDef)
}

impl FlagDef {
    pub fn validate(&self, boolean_flags: &IndexMap<String, bool>, counted_flags: &IndexMap<String, u64>) -> Result<(), String> {
        match self {
            FlagDef::BooleanFlagDef(bfd) => bfd.validate(&boolean_flags),
            FlagDef::CountedFlagDef(cfd) => cfd.validate(&counted_flags)
        }
    }


}

impl From<BooleanFlagDef> for FlagDef {
    fn from(item: BooleanFlagDef) -> Self {
        FlagDef::BooleanFlagDef(item)
    }
}

impl From<CountedFlagDef> for FlagDef {
    fn from(item: CountedFlagDef) -> Self {
        FlagDef::CountedFlagDef(item)
    }
}

#[derive(Builder, Clone, Derivative)]
#[derivative(Debug)]
pub struct SingleValueOptionDef {
    name: String,
    #[builder(default = "None")]
    short: Option<char>,
    #[builder(default = "None")]
    long: Option<String>,
    #[builder(default = "None")]
    help: Option<String>,
    #[builder(default = "false")]
    required: bool,
    #[builder(default = "vec![]")]
    #[derivative(Debug="ignore")]
    validators: Vec<ValueValidator>,
}

impl SingleValueOptionDef {

    pub fn get_help(&self) -> String {
        let mut s: String = String::new();
        s.push_str("    ");
        match (&self.short, &self.long) {
            (Some(short), None) => s.push_str(&format!("-{}", short)),
            (None, Some(long)) => s.push_str(&format!("--{}", long)),
            (Some(short), Some(long)) => s.push_str(&format!("-{}, --{}", short, long)),
            (None, None) => panic!("option {} must have either short or long value provided", self.name),
        }
        if let Some(help) = &self.help {
            s.push_str(&format!(" {}", &help));
        }
        s.push_str("\n");
        s
    }

    pub fn validate(&self, single_value_options: &IndexMap<String, String>) -> Result<(), String> {
        match &single_value_options.get(&self.name) {
            None => (),
            Some(ref value) => {
                for validator in &self.validators {
                    validator(&value)?;
                }
            }
        }
        Ok(())
    }


}

impl SingleValueOptionDefBuilder {

    pub fn new(name: String, short: Option<char>, long: Option<String>) -> SingleValueOptionDefBuilder {
        let mut svodb = SingleValueOptionDefBuilder::default();
        svodb.name(name)
            .short(short)
            .long(long);
        svodb
    }
}

#[derive(Builder, Clone, Derivative)]
#[derivative(Debug)]
pub struct MultiValueOptionDef {
    name: String,
    #[builder(default = "None")]
    short: Option<char>,
    #[builder(default = "None")]
    long: Option<String>,
    #[builder(default = "None")]
    help: Option<String>,
    #[builder(default = "0")]
    min_occurences: u64,
    #[builder(default = "None")]
    max_occurences: Option<u64>,
    #[builder(default = "vec![]")]
    #[derivative(Debug="ignore")]
    validators: Vec<MultiValueValidator>,
}

impl MultiValueOptionDef {

    pub fn get_help(&self) -> String {
        let mut s: String = String::new();
        s.push_str("    ");
        match (&self.short, &self.long) {
            (Some(short), None) => s.push_str(&format!("-{}", short)),
            (None, Some(long)) => s.push_str(&format!("--{}", long)),
            (Some(short), Some(long)) => s.push_str(&format!("-{}, --{}", short, long)),
            (None, None) => panic!("option {} must have either short or long value provided", self.name),
        }
        match &(self.min_occurences, self.max_occurences) {
            (v1, None) => {
                match v1 {
                    0 => s.push_str(&format!(" [{}] [...]", self.name.to_uppercase())),
                    _ => {
                        for _ in 0..*v1 { 
                            s.push_str(&format!(" {}", self.name.to_uppercase()))
                        }
                        s.push_str(" [...]");
                    }
                }
            },
            (v1, Some(v2)) => {
                for _ in 0..*v1 {
                    s.push_str(&format!(" {}", self.name.to_uppercase()))
                }
               
                if v1 < v2 {
                    for _ in *v1..*v2 {
                        s.push_str(&format!(" [{}]", self.name.to_uppercase()))
                    }
                }
            }
        }
        if let Some(help) = &self.help {
            s.push_str(&format!(" {}", &help));
        }
        s.push_str("\n");
        s
    }

    pub fn validate(&self, multi_value_options: &IndexMap<String, Vec<String>>) -> Result<(), String> {
        match &multi_value_options.get(&self.name) {
            None => (),
            Some(ref values) => {
                for validator in &self.validators {
                    validator(&values)?;
                }
            }
        }
        Ok(())
    }


}

impl MultiValueOptionDefBuilder {

    pub fn new(name: String, short: Option<char>, long: Option<String>) -> MultiValueOptionDefBuilder {
        let mut mvodb = MultiValueOptionDefBuilder::default();
        mvodb.name(name)
            .short(short)
            .long(long);
        mvodb
    }
}


#[derive(Clone, Debug)]
pub enum OptionDef {
    SingleValue(SingleValueOptionDef),
    MultiValue(MultiValueOptionDef),
}

impl OptionDef {
    pub fn name(&self) -> String {
        match self {
            OptionDef::SingleValue(o) => o.name.clone(),
            OptionDef::MultiValue(o) => o.name.clone(),
        }
    }

    pub fn get_help(&self) -> String {
        match self {
            OptionDef::SingleValue(o) => o.get_help(),
            OptionDef::MultiValue(o) => o.get_help()
        }
    }

    pub fn validate(&self, single_value_options: &IndexMap<String, String>, multi_value_options: &IndexMap<String, Vec<String>>) -> Result<(), String> {
        match self {
            OptionDef::SingleValue(o) => o.validate(&single_value_options),
            OptionDef::MultiValue(o) => o.validate(&multi_value_options)
        }
    }

    pub fn new_single_value(name: String, short: Option<char>, long: Option<String>) -> SingleValueOptionDefBuilder {
        SingleValueOptionDefBuilder::new(name, short, long)
    }

    pub fn new_multi_value(name: String, short: Option<char>, long: Option<String>) -> MultiValueOptionDefBuilder {
        MultiValueOptionDefBuilder::new(name, short, long)
    }
}

impl From<SingleValueOptionDef> for OptionDef {
    fn from(item: SingleValueOptionDef) -> Self {
        OptionDef::SingleValue(item)
    }
}

impl From<MultiValueOptionDef> for OptionDef {
    fn from(item: MultiValueOptionDef) -> Self {
        OptionDef::MultiValue(item)
    }
}


#[derive(Builder, Clone, Derivative)]
#[derivative(Debug)]
pub struct SingleValueArgumentDef {
    name: String,
    #[builder(default = "None")]
    help: Option<String>,
    #[builder(default = "false")]
    required: bool,
    #[builder(default = "vec![]")]
    #[derivative(Debug="ignore")]
    validators: Vec<ValueValidator>,
}

impl SingleValueArgumentDef {
    pub fn to_argument_def(self) -> ArgumentDef {
        ArgumentDef::SingleValue(self)
    }

    pub fn validate(&self, single_value_arguments: &IndexMap<String, String>) -> Result<(), String> {
        match &single_value_arguments.get(&self.name) {
            None => if self.required { Err(format!("required positional argument: {} is missing", self.name)) } else { Ok(())},
            Some(ref value) => {
                for validator in &self.validators {
                    validator(&value)?;
                }
                Ok(())
            }
        }
    }
}

impl SingleValueArgumentDefBuilder {

    pub fn new(name: String) -> SingleValueArgumentDefBuilder {
        let mut svadb = SingleValueArgumentDefBuilder::default();
        svadb.name(name);
        svadb
    }
}

#[derive(Builder, Clone, Derivative)]
#[derivative(Debug)]
pub struct MultiValueArgumentDef {
    name: String,
    #[builder(default = "None")]
    help: Option<String>,
    #[builder(default = "vec![]")]
    #[derivative(Debug="ignore")]
    validators: Vec<MultiValueValidator>,
    #[builder(default = "1")]
    min_occurences: u64,
    #[builder(default = "None")]
    max_occurences: Option<u64>,
}

impl MultiValueArgumentDef {
    pub fn to_argument_def(self) -> ArgumentDef {
        ArgumentDef::MultiValue(self)
    }

    pub fn validate(&self, multi_value_arguments: &IndexMap<String, Vec<String>>) -> Result<(), String> {
        match &multi_value_arguments.get(&self.name) {
            None => (),
            Some(ref values) => {
                for validator in &self.validators {
                    validator(&values)?;
                }
            }
        }
        Ok(())
    }
}

impl MultiValueArgumentDefBuilder {

    pub fn new(name: String) -> MultiValueArgumentDefBuilder {
        let mut mvadb = MultiValueArgumentDefBuilder::default();
        mvadb.name(name);
        mvadb
    }
}


#[derive(Clone, Debug)]
pub enum ArgumentDef {
    SingleValue(SingleValueArgumentDef),
    MultiValue(MultiValueArgumentDef),
}

impl ArgumentDef {

    pub fn new_single_value(name: String) -> SingleValueArgumentDefBuilder {
        SingleValueArgumentDefBuilder::new(name)
    }

    pub fn new_multi_value(name: String) -> MultiValueArgumentDefBuilder {
        MultiValueArgumentDefBuilder::new(name)
    }

    pub fn validate(&self, single_value_arguments: &IndexMap<String, String>, multi_value_arguments: &IndexMap<String, Vec<String>>) -> Result<(), String> {
        match self {
            ArgumentDef::SingleValue(arg) => arg.validate(&single_value_arguments),
            ArgumentDef::MultiValue(arg) => arg.validate(&multi_value_arguments)
        }
    }
}


impl From<SingleValueArgumentDef> for ArgumentDef {
    fn from(item: SingleValueArgumentDef) -> Self {
        ArgumentDef::SingleValue(item)
    }
}

impl From<MultiValueArgumentDef> for ArgumentDef {
    fn from(item: MultiValueArgumentDef) -> Self {
        ArgumentDef::MultiValue(item)
    }
}


#[derive(Builder, Clone)]
pub struct BasicSubCommandDef {
    name: String,
    #[builder(default = "None")]
    help: Option<String>,
    #[builder(default = "None")]
    validators: Option<Vec<SubCommandValidator>>,
    #[builder(default = "false")]
    required: bool,
    #[builder(default = "vec![]")]
    flags: Vec<FlagDef>,
    #[builder(default = "None")]
    flag_groups: Option<IndexMap<String, Group>>,
    #[builder(default = "OptionDefs(vec![])")]
    options: OptionDefs,
    #[builder(default = "None")]
    option_groups: Option<IndexMap<String, Group>>,
    #[builder(default = "ArgumentDefs(vec![])")]
    arguments: ArgumentDefs,
    subcommand: Option<Box<SubCommandDef>>,
    #[builder(default = "None")]
    subcommand_groups: Option<IndexMap<String, Group>>,
       
}




#[derive(Builder, Clone)]
pub struct SubCommandChainDef {
    name: String,
    #[builder(default = "None")]
    help: Option<String>,
    validators: Vec<SubCommandValidator>,
    arguments: Vec<ArgumentDef>,
    #[builder(default = "1")]
    min_occurences: u64,
    #[builder(default = "None")]
    max_occurences: Option<u64>,
    subcommand: Box<SubCommandDef>
}


#[derive(Builder, Clone)]
pub struct SubCommandChainsDef {
    name: String,
    help: Option<String>,
    validators: Vec<SubCommandValidator>,
    arguments: Vec<ArgumentDef>,
    subcommands: Vec<Box<SubCommandDef>>
}

#[derive(Clone)]
pub enum SubCommandDef {
    BasicSubCommandDef(BasicSubCommandDef),
    SubCommandChainDef(SubCommandChainDef),
    SubCommandChainsDef(SubCommandChainsDef),
    SubCommandEnumDef(Vec<Box<SubCommandDef>>)
}


#[derive(Builder, Clone, Default)]
pub struct AppDef {
    name: String,
    #[builder(default = "env!(\"CARGO_PKG_VERSION\").to_string()")]
    version: String,
    #[builder(default = "None")]
    help: Option<String>,
    #[builder(default = "vec![]")]
    validators: Vec<AppValidator>,
    #[builder(default = "FlagDefs(vec![])")]
    flags: FlagDefs,
    #[builder(default = "None")]
    flag_groups: Option<IndexMap<String, Group>>,
    #[builder(default = "OptionDefs(vec![])")]
    options: OptionDefs,
    #[builder(default = "None")]
    option_groups: Option<IndexMap<String, Group>>,
    #[builder(default = "ArgumentDefs(vec![])")]
    arguments: ArgumentDefs,
    #[builder(default = "None")]
    subcommand: Option<Box<SubCommandDef>>
}

impl AppDef {

    pub fn get_arguments_part(&self) -> String {
        let mut s = String::new();
        for arg in &self.arguments.0 {
            match arg {
                ArgumentDef::SingleValue(real_arg) => if real_arg.required {
                    s.push_str(&format!(" {}", real_arg.name))
                } else {
                    s.push_str(&format!(" [{}]", real_arg.name))
                },
                ArgumentDef::MultiValue(real_arg) => {
                    match (&real_arg.min_occurences, &real_arg.max_occurences) {
                        (v1, None) => {
                            match v1 {
                                0 => s.push_str(&format!(" [{}] [...]", real_arg.name)),
                                _ => {
                                    for _ in 0..*v1 { 
                                        s.push_str(&format!(" {}", real_arg.name))
                                    }
                                    s.push_str(" [...]");
                                }
                            }
                        },
                        (v1, Some(v2)) => {
                             for _ in 0..*v1 {
                                s.push_str(&format!(" {}", real_arg.name))
                            }
                           
                            if v1 < v2 {
                                for _ in *v1..*v2 {
                                    s.push_str(&format!(" [{}]", real_arg.name))
                                }
                            }
                        }
                    }
                }
            }
        }
        s
    }

    pub fn print_usage(&self){
        self.print_version();
        
        let mut usage = String::new();
         if let Some(help) = &self.help {
            usage.push_str(help);
        }
        usage.push_str("\n\n");
        usage.push_str("USAGE:\n    ");
 
        usage.push_str(&self.name.clone());
        if self.flags.0.len() > 0 {
            usage.push_str(" [FLAGS]");
        }
        if self.options.0.len() > 0 {
            usage.push_str(" [OPTIONS]");
        }

        if self.arguments.0.len() > 0 {
            usage.push_str(&self.get_arguments_part());
        }

        if let Some(_) = &self.subcommand {
            usage.push_str(" [SUBCOMMAND(S)]");
        }
        usage.push_str("\n\n");


        if self.options.0.len() > 0 {
            usage.push_str("OPTIONS:\n");
            let mut visited_options = IndexSet::new();
            if let Some(im) = &self.option_groups {
                for group in im.values() {
                    if let Some(help) = &group.help {
                        usage.push_str(&format!("\n    {}\n\n", help));
                    }
                    match &group.items {
                        None => { panic!("group is empty: {}", group.name); },
                        Some(items) => {
                            for option_name in items {
                                visited_options.insert(option_name);
                                match self.options.by_name(&option_name) {
                                    Some(option_def) => {
                                        usage.push_str(&option_def.get_help());
                                    },
                                    None => {
                                        panic!("Group {} doesn't have option {}", group.name, option_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            usage.push_str("\n");
            for option_def in &self.options.0 {
                if !visited_options.contains(&option_def.name()) {
                    usage.push_str(&option_def.get_help());
                }
            }
        }


        println!("{}", usage);
    }

    pub fn print_version(&self) {
        println!("{} {}", self.name, self.version);
    }

    pub fn parse_args(&self, args: &[&str]) -> Result<App, String> {
        #[derive(Debug)]
        enum State<'a> {
            ExpectAnything,
            ExpectOptionValue(&'a OptionDef),
            ExpectArgumentValue(&'a ArgumentDef, usize),
            ExpectCommand,
            ExpectEnd
        };
        let mut app = App::new(&self.name);
         
        let mut state = State::ExpectAnything;
        let parser = parser::Parser::new(&args);

        for token in parser.iter() {
            match (&mut state, &token) {

                (State::ExpectAnything, parser::Token::Short(ch)) => {
                    match self.flags.by_short(ch) {
                        Some(ref flag_def) => {
                            match flag_def {
                                FlagDef::BooleanFlagDef(bf) => {
                                    match app.boolean_flags.get(&bf.name) {
                                        Some(_) => return Err("Invalid arguments".to_string()),
                                        None => { app.boolean_flags.insert(bf.name.clone(), true);},
                                    };
                                    state = State::ExpectAnything;

                                },
                                FlagDef::CountedFlagDef(cf) => {
                                    match app.counted_flags.entry(cf.name.clone()) {
                                        Entry::Occupied(mut value) => { let v = value.get_mut(); *v+=1;},
                                        Entry::Vacant(entry) => {entry.insert(1); }
                                    }
                                    state = State::ExpectAnything;
                                },

                            }
                        },
                        None => {
                            match self.options.by_short(ch) {
                                Some(ref option_def) => {
                                    state = State::ExpectOptionValue(option_def);
                                },
                                None => return Err("Invalid arguments".to_string())
                            }
                        
                        }
                    }
                },
                (State::ExpectAnything, parser::Token::Long(param)) => {
                    match self.flags.by_long(param) {
                        Some(ref flag_def) => {
                            match flag_def {
                                FlagDef::BooleanFlagDef(bf) => {
                                    match app.boolean_flags.get(&bf.name) {
                                        Some(_) => return Err("Invalid arguments".to_string()),
                                        None => { app.boolean_flags.insert(bf.name.clone(), true);},
                                    };
                                    state = State::ExpectAnything;

                                },
                                FlagDef::CountedFlagDef(cf) => {
                                    match app.counted_flags.entry(cf.name.clone()) {
                                        Entry::Occupied(mut value) => { let v = value.get_mut(); *v+=1;},
                                        Entry::Vacant(entry) => {entry.insert(1); }
                                    }
                                    state = State::ExpectAnything;
                                },

                            }
                        },
                        None => {
                            match self.options.by_long(param) {
                                Some(ref option_def) => {
                                    state = State::ExpectOptionValue(option_def);
                                },
                                None => return Err("Invalid arguments".to_string())
                            }
                        
                        }
                    }
                },

                (State::ExpectOptionValue(ref option_def), parser::Token::Value(value)) => {
                    match option_def {
                        OptionDef::SingleValue(svod) => {
                            match app.single_value_arguments.get(&svod.name) {
                                Some(_) => return Err("Invalid arguments".to_string()),
                                None => {
                                    app.single_value_arguments.insert(svod.name.clone(), value.to_string());
                                    state = State::ExpectAnything;
                                }
                            }
                        },
                        OptionDef::MultiValue(mvod) => {
                            match app.multi_value_arguments.entry(mvod.name.clone()) {
                                Entry::Occupied(mut entry) => {
                                    let v = entry.get_mut();
                                    v.push(value.to_string());
                                },
                                Entry::Vacant(entry) => {
                                    entry.insert(vec![value.to_string()]);
                                }
                            }
                            
                             state = State::ExpectAnything;
                        },
                    }
                },

                (State::ExpectAnything, parser::Token::Value(value)) => {
                    if self.arguments.0.len() > 0 {
                        let arg_def = &self.arguments.0[0];
                        match arg_def {
                            ArgumentDef::SingleValue(svad) => {
                                app.single_value_arguments.insert(svad.name.clone(), value.to_string());
                                if self.arguments.0.len() == 1 {
                                    if self.subcommand.is_some() {
                                        state = State::ExpectCommand;
                                    } else {
                                        state = State::ExpectEnd;
                                    }
                                } else {
                                    state = State::ExpectArgumentValue(&self.arguments.0[1], 1);
                                }
                            },
                            ArgumentDef::MultiValue(mvad) => {
                                app.multi_value_arguments.insert(mvad.name.clone(), vec![value.to_string()]);

                                if mvad.max_occurences == Some(1) {
                                    if self.subcommand.is_some() {
                                        state = State::ExpectCommand;
                                    } else {
                                        state = State::ExpectEnd;
                                    }
                                } else {
                                    state = State::ExpectArgumentValue(&self.arguments.0[1], 1);
                                }
                            },
                        }
                    }
                },
                
                (State::ExpectArgumentValue(arg_def, arg_idx), parser::Token::Value(value)) => {
                    match arg_def {
                        ArgumentDef::SingleValue(svad) => {
                            app.single_value_arguments.insert(svad.name.clone(), value.to_string());
                            if *arg_idx + 1 == self.arguments.0.len() {
                                if self.subcommand.is_some() {
                                    state = State::ExpectCommand;
                                } else {
                                    state = State::ExpectEnd;
                                }
                            } else {
                                state = State::ExpectArgumentValue(&self.arguments.0[*arg_idx + 1], *arg_idx + 1);
                            }
                        },
                        ArgumentDef::MultiValue(mvad) => {
                            
                            let value_cnt = match app.multi_value_arguments.entry(mvad.name.clone()) {
                                Entry::Occupied(mut entry) => {
                                    let values = entry.get_mut();
                                    values.push(value.to_string());
                                    values.len()
                                },
                                Entry::Vacant(entry) => {
                                    entry.insert(vec![value.to_string()]);
                                    1
                                }
                            };
                            if let Some(max) = mvad.max_occurences {
                                if value_cnt == max as usize {
                                    if *arg_idx + 1 == self.arguments.0.len() {
                                        if self.subcommand.is_some() {
                                            state = State::ExpectCommand;
                                        } else {
                                            state = State::ExpectEnd;
                                        }
                                    } else {
                                        state = State::ExpectArgumentValue(&self.arguments.0[*arg_idx + 1], *arg_idx + 1);
                                    }
                                }
                            }
                        }
                    }
                },
                (State::ExpectAnything, parser::Token::End) | (State::ExpectEnd, parser::Token::End) => {
                    //TODO: validate arguments
                    self.validate(&app)?;
                    return Ok(app);
                },
                (state, token) => return Err(format!("Invalid arguments: {:?}, {:?}", state, token ))
            }
        };
        
        Err("err".to_string())
    }

    pub fn from_args(&self, args: &[&str]) -> Result<App, String> {
        match self.parse_args(&args) {
            Ok(app) => Ok(app),
            Err(s) => {
                eprintln!("{}", s);
                std::process::exit(1);
            }
        }
    }

    fn validate(&self, app: &App) -> Result<(), String> {

        self.flags.validate(&app.boolean_flags, &app.counted_flags)?;
        self.options.validate(&app.single_value_options, &app.multi_value_options)?;
        self.arguments.validate(&app.single_value_arguments, &app.multi_value_arguments)?;

        for validator in &self.validators {
            validator(&app)?;
        }

        Ok(())
    }

}

impl AppDefBuilder {

    pub fn new(name: String) -> AppDefBuilder {
        let mut app_def_builder = AppDefBuilder::default();
        app_def_builder.add_flag(
            FlagDef::BooleanFlagDef(
                BooleanFlagDefBuilder::new("verbose".to_string(), Some('V'), Some("verbose".to_string()))
                .build()
                .unwrap()
            )
        );
        app_def_builder.name(name);
        app_def_builder
    }

    pub fn add_flag(&mut self, flag_def: FlagDef) -> &mut Self {
        match &mut self.flags {
            Some(ref mut flag_defs) => { flag_defs.add_flag(flag_def); },
            None => { self.flags = Some(FlagDefs(vec![flag_def])); }
        }
        self
    }

    pub fn add_flag_group(&mut self, group: Group) -> Result<&mut Self, String> {
        match &mut self.flag_groups {
            None | Some(None) => {
                let mut hm = IndexMap::new();
                hm.insert(group.name.clone(), group);
                self.flag_groups = Some(Some(hm));
                Ok(self)
            },
            Some(Some(ref mut hm)) => {
                hm.insert(group.name.clone(), group);
                Ok(self)
            }
        }
    }

    pub fn add_option_group(&mut self, group: Group) -> Result<&mut Self, String> {
        match &mut self.option_groups {
            None | Some(None) => {
                let mut hm = IndexMap::new();
                hm.insert(group.name.clone(), group);
                self.option_groups = Some(Some(hm));
                Ok(self)
            },
            Some(Some(ref mut hm)) => {
                hm.insert(group.name.clone(), group);
                Ok(self)
            }
        }
    }

    pub fn add_option_groups(&mut self, groups: Vec<(Group, Vec<OptionDef>)>) -> Result<&mut Self, String> {
        for (mut group, defs) in groups {
            match &mut self.option_groups {
                None | Some(None) => {
                    let mut im = IndexMap::new();
                    for def in defs {
                        group.add_item(&def.name());
                        self.add_option(def);
                    }
                    im.insert(group.name.clone(), group);
                    self.option_groups = Some(Some(im));

                },
                Some(Some(ref mut im)) => {
                    for def in &defs {
                        group.add_item(&def.name());
                    }
                    im.insert(group.name.clone(), group);
                    for def in defs {
                        self.add_option(def);
                    }
                }
            }
        }
        Ok(self)
    }


    pub fn add_option(&mut self, option_def: OptionDef) -> &mut Self {
        match &mut self.options {
            Some(v) => v.0.push(option_def),
            None => self.options = Some(OptionDefs(vec![option_def]))
        };
        self
    }

    pub fn add_option_to_group(&mut self, group_name: &str, option_def: OptionDef) -> Result<&mut Self, String> {
    if let Some(Some(ref mut hm)) = self.option_groups {

        match hm.entry(group_name.to_string()){
            Entry::Occupied(mut g)  => {
                let mut group = g.get_mut();
                match &mut group.items {
                    None => {
                        let mut hs = IndexSet::new();
                        let s:String = (&option_def.name()).clone();
                        hs.insert(s);
                        group.items = Some(hs);
                    },
                    Some(v) => {
                        v.insert(option_def.name().clone());
                    }
                }
            },
            Entry::Vacant(_)  => {},
        }

    } else {
        return Err(format!("group does not exist: {}", group_name))
    }
        match &mut self.options {
            Some(v) => v.0.push(option_def),
            None => self.options = Some(OptionDefs(vec![option_def]))
        };
        Ok(self)
    }


    pub fn add_argument(&mut self, argument_def: ArgumentDef) -> &mut Self {
        match &mut self.arguments {
            Some(v) => v.0.push(argument_def),
            None => self.arguments = Some(ArgumentDefs(vec![argument_def]))
        };
        self
    }
}

#[derive(Debug)]
pub struct App {
    pub name: String,
    pub boolean_flags: IndexMap<String, bool>,
    pub counted_flags: IndexMap<String, u64>,
    pub single_value_options: IndexMap<String, String>,
    pub multi_value_options: IndexMap<String, Vec<String>>,
    pub single_value_arguments: IndexMap<String, String>,
    pub multi_value_arguments: IndexMap<String, Vec<String>>,
}

impl App {
    pub fn new(name: &str) -> App {
        App {
            name: name.to_string(),
            boolean_flags: IndexMap::new(),
            counted_flags: IndexMap::new(),
            single_value_options: IndexMap::new(),
            multi_value_options: IndexMap::new(),
            single_value_arguments: IndexMap::new(),
            multi_value_arguments: IndexMap::new(),
        }
    }
}
