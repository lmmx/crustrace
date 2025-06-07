//! Declarative parsing types using unsynn

use unsynn::*;

keyword! {
    /// The "level" keyword
    pub KLevel = "level";
    /// The "name" keyword
    pub KName = "name";
}

operator! {
    /// The "=" operator
    pub Eq = "=";
}

unsynn! {
    /// Declarative instrument arguments structure
    pub struct InstrumentInner {
        /// Comma-delimited list of arguments
        pub args: Option<CommaDelimitedVec<InstrumentArg>>,
    }

    /// Single instrument argument
    pub enum InstrumentArg {
        /// level = "debug"
        Level(LevelArg),
        /// name = "custom"
        Name(NameArg),
    }

    /// Level argument: level = "debug"
    pub struct LevelArg {
        pub _level: KLevel,
        pub _eq: Eq,
        pub value: LiteralString,
    }

    /// Name argument: name = "custom"
    pub struct NameArg {
        pub _name: KName,
        pub _eq: Eq,
        pub value: LiteralString,
    }
}
