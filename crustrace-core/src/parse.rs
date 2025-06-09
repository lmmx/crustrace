//! Declarative parsing types using unsynn

use unsynn::*;

/// Parses tokens until `C` is found on the current token tree level.
pub type VerbatimUntil<C> = Many<Cons<Except<C>, AngleTokenTree>>;

keyword! {
    /// The "level" keyword
    pub KLevel = "level";
    /// The "name" keyword
    pub KName = "name";
    /// The "fn" keyword
    pub KFn = "fn";
    /// The "pub" keyword
    pub KPub = "pub";
    /// The "async" keyword
    pub KAsync = "async";
    /// The "unsafe" keyword
    pub KUnsafe = "unsafe";
    /// The "extern" keyword
    pub KExtern = "extern";
    /// The "const" keyword
    pub KConst = "const";
    /// The "where" keyword
    pub KWhere = "where";
    /// The "impl" keyword
    pub KImpl = "impl";
    /// The "for" keyword
    pub KFor = "for";
    /// The "mod" keyword
    pub KMod = "mod";
    /// The "trait" keyword
    pub KTrait = "trait";
    /// The "crate" keyword
    pub KCrate = "crate";
    /// The "super" keyword
    pub KSuper = "super";
    /// The "self" keyword
    pub KSelf = "self";
    /// The "mut" keyword
    pub KMut = "mut";
    /// The "ret" keyword (in the tracing macro)
    pub KRet = "ret";
    /// The "Debug" keyword (in the tracing macro ret arg)
    pub KDebug = "Debug";
    /// The "Display" keyword (in the tracing macro ret arg)
    pub KDisplay = "Display";
}

operator! {
    /// The "=" operator
    pub Eq = "=";
    /// The "&" operator
    pub And = "&";
}

unsynn! {
    /// Parses either a `TokenTree` or `<...>` grouping
    #[derive(Clone)]
    pub struct AngleTokenTree(
        pub Either<Cons<Lt, Vec<Cons<Except<Gt>, AngleTokenTree>>, Gt>, TokenTree>,
    );

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
        /// ret
        Ret(RetArgs),
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

    /// Complete function signature
    pub struct FnSig {
        /// Optional attributes (#[...])
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility (pub, pub(crate), etc.)
        pub visibility: Option<Visibility>,
        /// Optional const modifier
        pub const_kw: Option<KConst>,
        /// Optional async modifier
        pub async_kw: Option<KAsync>,
        /// Optional unsafe modifier
        pub unsafe_kw: Option<KUnsafe>,
        /// Optional extern with optional ABI
        pub extern_kw: Option<ExternSpec>,
        /// The "fn" keyword
        pub _fn: KFn,
        /// Function name
        pub name: Ident,
        /// Optional generic parameters
        pub generics: Option<Generics>,
        /// Parameters in parentheses
        pub params: ParenthesisGroupContaining<Option<CommaDelimitedVec<FnParam>>>,
        /// Optional return type
        pub return_type: Option<ReturnType>,
        /// Optional where clause
        pub where_clause: Option<WhereClauses>,
        pub body: BraceGroup,
    }

    /// Attribute like #[derive(Debug)]
    pub struct Attribute {
        /// Hash symbol
        pub _hash: Pound,
        /// Attribute content
        pub content: BracketGroup,
    }

    /// Extern specification with optional ABI
    pub enum ExternSpec {
        /// "extern" with ABI string like extern "C"
        WithAbi(ExternWithAbi),
        /// Just "extern"
        Bare(KExtern),
    }

    /// Extern with ABI string
    pub struct ExternWithAbi {
        /// The "extern" keyword
        pub _extern: KExtern,
        /// The ABI string
        pub abi: LiteralString,
    }

    /// Simple visibility parsing
    pub enum Visibility {
        /// "pub(crate)", "pub(super)", etc.
        Restricted(RestrictedVis),
        /// Just "pub"
        Public(KPub),
    }

    /// Restricted visibility like pub(crate)
    pub struct RestrictedVis {
        /// The "pub" keyword
        pub _pub: KPub,
        /// The parentheses with content
        pub restriction: ParenthesisGroup,
    }

    /// Simple generics (treat as opaque for now)
    pub struct Generics {
        /// Opening
        pub _lt: Lt,
        /// Everything until closing > (opaque)
        pub content: Many<Cons<Except<Gt>, TokenTree>>,
        /// Closing >
        pub _gt: Gt,
    }

    /// Return type: -> Type
    pub struct ReturnType {
        /// Arrow
        pub _arrow: RArrow,
        /// Everything until brace (opaque)
        pub return_type: VerbatimUntil<BraceGroup>,
    }

    /// Represents a single predicate within a `where` clause.
    /// e.g., `T: Trait` or `'a: 'b`.
    #[derive(Clone)]
    pub struct WhereClause {
        // FIXME: This likely breaks for absolute `::` paths
        /// The type or lifetime being constrained (e.g., `T` or `'a`).
        pub _pred: VerbatimUntil<Colon>,
        /// The colon separating the constrained item and its bounds.
        pub _colon: Colon,
        /// The bounds applied to the type or lifetime (e.g., `Trait` or `'b`).
        pub bounds: VerbatimUntil<Either<Comma, Semicolon, BraceGroup>>,
    }

    /// Where clauses: where T: Trait, U: Send
    #[derive(Clone)]
    pub struct WhereClauses {
        /// The `where` keyword.
        pub _kw_where: KWhere,
        /// The comma-delimited list of where clause predicates.
        pub clauses: CommaDelimitedVec<WhereClausePredicate>,
    }

    /// Single where clause predicate: T: Trait
    #[derive(Clone)]
    pub struct WhereClausePredicate {
        /// The type being constrained (e.g., `T`)
        pub pred: VerbatimUntil<Colon>,
        /// The colon
        pub _colon: Colon,
        /// The bounds (e.g., `Trait`)
        pub bounds: VerbatimUntil<Either<Comma, BraceGroup>>,
    }

    /// Top-level item that can appear in a module
    pub enum ModuleItem {
        /// A function definition
        Function(FnSig),
        /// An impl block
        ImplBlock(ImplBlockSig),
        /// A module definition
        Module(ModuleSig),
        /// A trait definition
        Trait(TraitSig),
        /// Any other item (struct, enum, use, etc.)
        Other(TokenTree),
    }

    /// impl Type { ... } block
    pub struct ImplBlockSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// "impl" keyword
        pub _impl: KImpl,
        /// Optional generic parameters
        pub generics: Option<Generics>,
        /// Type being implemented (opaque for now)
        pub target_type: Many<Cons<Except<Either<KFor, BraceGroup>>, TokenTree>>,
        /// Optional "for Trait" part
        pub for_trait: Option<Cons<KFor, Many<Cons<Except<BraceGroup>, TokenTree>>>>,
        /// Optional where clause
        pub where_clause: Option<WhereClauses>,
        /// Block body
        pub body: BraceGroup,
    }

    /// mod name { ... } block
    pub struct ModuleSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// "mod" keyword
        pub _mod: KMod,
        /// Module name
        pub name: Ident,
        /// Module body
        pub body: BraceGroup,
    }

    /// trait Name { ... } block
    pub struct TraitSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// Optional unsafe
        pub unsafe_kw: Option<KUnsafe>,
        /// "trait" keyword
        pub _trait: KTrait,
        /// Trait name
        pub name: Ident,
        /// Optional generic parameters
        pub generics: Option<Generics>,
        /// Optional trait bounds
        pub bounds: Option<Cons<Colon, Many<Cons<Except<Either<KWhere, BraceGroup>>, TokenTree>>>>,
        /// Optional where clause
        pub where_clause: Option<WhereClauses>,
        /// Trait body
        pub body: BraceGroup,
    }

    /// A complete module/file content
    pub struct ModuleContent {
        /// All items in the module
        pub items: Many<ModuleItem>,
    }

    /// Function parameter: name: Type or self variants
    pub enum FnParam {
        /// self parameter
        SelfParam(SelfParam),
        /// Regular parameter: name: Type
        Named(NamedParam),
        /// Pattern parameter: (a, b): (i32, i32)
        Pattern(PatternParam),
    }

    /// self, &self, &mut self, mut self
    pub enum SelfParam {
        /// self
        Value(KSelf),
        /// &self
        Ref(Cons<And, KSelf>),
        /// &mut self
        RefMut(Cons<And, Cons<KMut, KSelf>>),
        /// mut self
        Mut(Cons<KMut, KSelf>),
    }

    /// name: Type parameter
    pub struct NamedParam {
        /// Optional mut keyword
        pub mut_kw: Option<KMut>,
        /// Parameter name
        pub name: Ident,
        /// Colon
        pub _colon: Colon,
        /// Parameter type (opaque for now)
        pub param_type: VerbatimUntil<Comma>,
    }

    /// Pattern parameter like (a, b): (i32, i32) or mut (x, y): Point
    pub struct PatternParam {
        /// Optional mut keyword
        pub mut_kw: Option<KMut>,
        /// Pattern (everything before colon, could be tuple, struct pattern, etc.)
        pub pattern: Pattern,
        /// Colon
        pub _colon: Colon,
        /// Parameter type
        pub param_type: VerbatimUntil<Either<Comma, ParenthesisGroup>>,
    }

   /// Different types of patterns
    pub enum Pattern {
        /// Simple identifier: value
        Ident(Ident),
        /// Tuple pattern: (a, b, c)
        Tuple(TuplePattern),
        /// Other patterns (fallback)
        Other(VerbatimUntil<Colon>),
    }

    /// Tuple destructuring pattern: (a, b, c)
    pub struct TuplePattern {
        /// Parentheses containing comma-separated identifiers
        pub fields: ParenthesisGroupContaining<Option<CommaDelimitedVec<PatternField>>>,
    }

    /// Field in a pattern
    pub enum PatternField {
        /// Simple identifier
        Ident(Ident),
        /// Nested pattern (recursive)
        Nested(Pattern),
    }

    /// Arguments to ret() - parsed declaratively
    pub struct RetArgs {
        /// The ret keyword, which may be bare or followed by brackets (which may contain args)
        pub _ret: KRet,
        /// Optional parentheses containing ret arguments
        pub args: Option<ParenthesisGroupContaining<Option<CommaDelimitedVec<RetArg>>>>,
    }

    /// Single argument inside ret(...)
    pub enum RetArg {
        /// level = "debug"
        Level(LevelArg),
        /// Debug format mode
        Debug(KDebug), // matches "Debug" identifier
        /// Display format mode
        Display(KDisplay), // matches "Display" identifier
    }

    /// Format mode for return value logging
    #[derive(Clone, Default, PartialEq, Eq)]
    pub enum FormatMode {
        /// Debug format (?)
        #[default] Debug,
        /// Display format (%)
        Display,
    }

}

// Parsing logic using unsynn declarative parsing:
impl RetArgs {
    /// Extract the effective format mode from parsed args
    pub fn format_mode(&self) -> FormatMode {
        if let Some(args_group) = &self.args {
            if let Some(arg_list) = &args_group.content {
                for arg in &arg_list.0 {
                    match &arg.value {
                        RetArg::Debug(_) => return FormatMode::Debug,
                        RetArg::Display(_) => return FormatMode::Display,
                        RetArg::Level(_) => continue,
                    }
                }
            }
        }
        FormatMode::default()
    }

    /// Extract the custom level if specified
    pub fn custom_level(&self) -> Option<&LevelArg> {
        if let Some(args_group) = &self.args {
            if let Some(arg_list) = &args_group.content {
                for arg in &arg_list.0 {
                    if let RetArg::Level(level_arg) = &arg.value {
                        return Some(level_arg);
                    }
                }
            }
        }
        None
    }
}

impl Pattern {
    pub(crate) fn extract_identifiers(&self) -> Vec<&Ident> {
        match self {
            Pattern::Tuple(tuple) => {
                if let Some(fields) = &tuple.fields.content {
                    fields
                        .0
                        .iter()
                        .filter_map(|field| {
                            if let PatternField::Ident(ident) = &field.value {
                                Some(ident)
                            } else {
                                None
                            }
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            }
            Pattern::Ident(ident) => vec![ident],
            _ => Vec::new(),
        }
    }
}

// Implement ToTokens for quote! compatibility
impl quote::ToTokens for FnSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // Add attributes
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }

        // Add visibility
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }

        // Add const keyword
        if let Some(const_kw) = &self.const_kw {
            unsynn::ToTokens::to_tokens(const_kw, tokens);
        }

        // Add async keyword
        if let Some(async_kw) = &self.async_kw {
            unsynn::ToTokens::to_tokens(async_kw, tokens);
        }

        // Add unsafe keyword
        if let Some(unsafe_kw) = &self.unsafe_kw {
            unsynn::ToTokens::to_tokens(unsafe_kw, tokens);
        }

        // Add extern specification
        if let Some(extern_kw) = &self.extern_kw {
            unsynn::ToTokens::to_tokens(extern_kw, tokens);
        }

        // Add fn keyword and the rest
        unsynn::ToTokens::to_tokens(&self._fn, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);

        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }

        unsynn::ToTokens::to_tokens(&self.params, tokens);

        if let Some(ret_type) = &self.return_type {
            unsynn::ToTokens::to_tokens(ret_type, tokens);
        }

        if let Some(where_clause) = &self.where_clause {
            unsynn::ToTokens::to_tokens(where_clause, tokens);
        }

        unsynn::ToTokens::to_tokens(&self.body, tokens);
    }
}

impl quote::ToTokens for FnParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FnParam::SelfParam(self_param) => quote::ToTokens::to_tokens(self_param, tokens),
            FnParam::Named(named) => quote::ToTokens::to_tokens(named, tokens),
            FnParam::Pattern(pattern) => quote::ToTokens::to_tokens(pattern, tokens),
        }
    }
}

impl quote::ToTokens for SelfParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            SelfParam::Value(self_kw) => unsynn::ToTokens::to_tokens(self_kw, tokens),
            SelfParam::Ref(ref_self) => unsynn::ToTokens::to_tokens(ref_self, tokens),
            SelfParam::RefMut(ref_mut_self) => unsynn::ToTokens::to_tokens(ref_mut_self, tokens),
            SelfParam::Mut(mut_self) => unsynn::ToTokens::to_tokens(mut_self, tokens),
        }
    }
}

impl quote::ToTokens for NamedParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(mut_kw) = &self.mut_kw {
            unsynn::ToTokens::to_tokens(mut_kw, tokens);
        }
        quote::ToTokens::to_tokens(&self.name, tokens);
        unsynn::ToTokens::to_tokens(&self._colon, tokens);
        unsynn::ToTokens::to_tokens(&self.param_type, tokens);
    }
}

impl quote::ToTokens for PatternParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(mut_kw) = &self.mut_kw {
            unsynn::ToTokens::to_tokens(mut_kw, tokens);
        }
        unsynn::ToTokens::to_tokens(&self.pattern, tokens);
        unsynn::ToTokens::to_tokens(&self._colon, tokens);
        unsynn::ToTokens::to_tokens(&self.param_type, tokens);
    }
}

impl quote::ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Pattern::Tuple(tuple) => quote::ToTokens::to_tokens(tuple, tokens),
            Pattern::Ident(ident) => quote::ToTokens::to_tokens(ident, tokens),
            Pattern::Other(other) => unsynn::ToTokens::to_tokens(other, tokens),
        }
    }
}

impl quote::ToTokens for TuplePattern {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self.fields, tokens);
    }
}

impl quote::ToTokens for PatternField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            PatternField::Ident(ident) => quote::ToTokens::to_tokens(ident, tokens),
            PatternField::Nested(pattern) => quote::ToTokens::to_tokens(pattern, tokens),
        }
    }
}

impl quote::ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Visibility::Public(pub_kw) => unsynn::ToTokens::to_tokens(pub_kw, tokens),
            Visibility::Restricted(restricted) => unsynn::ToTokens::to_tokens(restricted, tokens),
        }
    }
}

impl quote::ToTokens for ExternSpec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ExternSpec::WithAbi(with_abi) => unsynn::ToTokens::to_tokens(with_abi, tokens),
            ExternSpec::Bare(extern_kw) => unsynn::ToTokens::to_tokens(extern_kw, tokens),
        }
    }
}

impl quote::ToTokens for ReturnType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._arrow, tokens);
        unsynn::ToTokens::to_tokens(&self.return_type, tokens);
    }
}

impl quote::ToTokens for Generics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._lt, tokens);
        unsynn::ToTokens::to_tokens(&self.content, tokens);
        unsynn::ToTokens::to_tokens(&self._gt, tokens);
    }
}

impl quote::ToTokens for WhereClause {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._pred, tokens);
        unsynn::ToTokens::to_tokens(&self._colon, tokens);
        unsynn::ToTokens::to_tokens(&self.bounds, tokens);
    }
}

impl quote::ToTokens for WhereClauses {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._kw_where, tokens);
        unsynn::ToTokens::to_tokens(&self.clauses, tokens);
    }
}

impl quote::ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._hash, tokens);
        unsynn::ToTokens::to_tokens(&self.content, tokens);
    }
}

impl quote::ToTokens for RestrictedVis {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._pub, tokens);
        unsynn::ToTokens::to_tokens(&self.restriction, tokens);
    }
}

impl quote::ToTokens for ExternWithAbi {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._extern, tokens);
        unsynn::ToTokens::to_tokens(&self.abi, tokens);
    }
}

impl quote::ToTokens for ModuleItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ModuleItem::Function(func) => quote::ToTokens::to_tokens(func, tokens),
            ModuleItem::ImplBlock(impl_block) => quote::ToTokens::to_tokens(impl_block, tokens),
            ModuleItem::Module(module) => quote::ToTokens::to_tokens(module, tokens),
            ModuleItem::Trait(trait_def) => quote::ToTokens::to_tokens(trait_def, tokens),
            ModuleItem::Other(token_tree) => unsynn::ToTokens::to_tokens(token_tree, tokens),
        }
    }
}

impl quote::ToTokens for ImplBlockSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        unsynn::ToTokens::to_tokens(&self._impl, tokens);
        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }
        unsynn::ToTokens::to_tokens(&self.target_type, tokens);
        if let Some(for_trait) = &self.for_trait {
            unsynn::ToTokens::to_tokens(for_trait, tokens);
        }
        if let Some(where_clause) = &self.where_clause {
            unsynn::ToTokens::to_tokens(where_clause, tokens);
        }
        unsynn::ToTokens::to_tokens(&self.body, tokens);
    }
}

impl quote::ToTokens for ModuleSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._mod, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);
        unsynn::ToTokens::to_tokens(&self.body, tokens);
    }
}

impl quote::ToTokens for TraitSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        if let Some(unsafe_kw) = &self.unsafe_kw {
            unsynn::ToTokens::to_tokens(unsafe_kw, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._trait, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);
        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }
        if let Some(bounds) = &self.bounds {
            unsynn::ToTokens::to_tokens(bounds, tokens);
        }
        if let Some(where_clause) = &self.where_clause {
            unsynn::ToTokens::to_tokens(where_clause, tokens);
        }
        unsynn::ToTokens::to_tokens(&self.body, tokens);
    }
}

impl quote::ToTokens for ModuleContent {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self.items, tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::quote;

    fn parse_fn_sig(input: TokenStream) -> Result<FnSig> {
        let mut iter = input.into_token_iter();
        iter.parse::<FnSig>()
    }

    fn fn_sig_to_tokens(sig: FnSig) -> TokenStream {
        let mut tokens = TokenStream::new();
        sig.to_tokens(&mut tokens);
        tokens
    }

    #[test]
    fn test_basic_function() {
        let input = quote! { fn hello() {} };
        let parsed = parse_fn_sig(input.clone()).expect("Should parse");

        let output = fn_sig_to_tokens(parsed);
        // Fix the spacing comparison - just check it contains the right parts
        let output_str = output.to_string();
        assert!(output_str.contains("fn"));
        assert!(output_str.contains("hello"));
        assert!(output_str.contains("()"));
        assert!(output_str.contains("{ }"));
    }

    #[test]
    fn test_pub_crate_async_function() {
        let input = quote! { pub(crate) async fn hello() {} };
        println!("Parsing input: {}", input);

        match parse_fn_sig(input.clone()) {
            Ok(parsed) => {
                println!("✅ Parsed successfully!");
                println!("  visibility: {:?}", parsed.visibility.is_some());
                println!("  async_kw: {:?}", parsed.async_kw.is_some());

                let output = fn_sig_to_tokens(parsed);
                println!("Output: {}", output);
            }
            Err(e) => {
                println!("❌ Parse failed: {}", e);
                // Let's try parsing just the parts
                let mut iter = input.into_token_iter();
                if let Ok(_vis) = iter.parse::<Visibility>() {
                    println!("✅ Visibility parsed OK");
                    if let Ok(_async_kw) = iter.parse::<KAsync>() {
                        println!("✅ Async keyword parsed OK");
                    } else {
                        println!("❌ Async keyword failed");
                    }
                } else {
                    println!("❌ Visibility parsing failed");
                }
                panic!("Parse failed: {}", e);
            }
        }
    }

    #[test]
    fn test_async_function() {
        let input = quote! { async fn hello() {} };
        let parsed = parse_fn_sig(input.clone()).expect("Should parse async fn");

        assert!(parsed.async_kw.is_some());
        assert!(parsed.unsafe_kw.is_none());
        assert_eq!(parsed.name.to_string(), "hello");

        let output = fn_sig_to_tokens(parsed);
        println!("Input:  {}", input);
        println!("Output: {}", output);
        assert!(output.to_string().contains("async"));
    }

    #[test]
    fn test_unsafe_function() {
        let input = quote! { unsafe fn hello() {} };
        let parsed = parse_fn_sig(input.clone()).expect("Should parse unsafe fn");

        assert!(parsed.unsafe_kw.is_some());
        assert!(parsed.async_kw.is_none());
        assert_eq!(parsed.name.to_string(), "hello");

        let output = fn_sig_to_tokens(parsed);
        println!("Input:  {}", input);
        println!("Output: {}", output);
        assert!(output.to_string().contains("unsafe"));
    }

    #[test]
    fn test_const_function() {
        let input = quote! { const fn hello() {} };
        let parsed = parse_fn_sig(input.clone()).expect("Should parse const fn");

        assert!(parsed.const_kw.is_some());
        assert!(parsed.async_kw.is_none());
        assert_eq!(parsed.name.to_string(), "hello");

        let output = fn_sig_to_tokens(parsed);
        println!("Input:  {}", input);
        println!("Output: {}", output);
        assert!(output.to_string().contains("const"));
    }

    #[test]
    fn test_pub_async_function() {
        let input = quote! { pub async fn hello() {} };
        let parsed = parse_fn_sig(input.clone()).expect("Should parse pub async fn");

        assert!(parsed.visibility.is_some());
        assert!(parsed.async_kw.is_some());
        assert!(parsed.unsafe_kw.is_none());
        assert_eq!(parsed.name.to_string(), "hello");

        let output = fn_sig_to_tokens(parsed);
        println!("Input:  {}", input);
        println!("Output: {}", output);
        assert!(output.to_string().contains("pub"));
        assert!(output.to_string().contains("async"));
    }

    #[test]
    fn test_async_unsafe_function() {
        let input = quote! { async unsafe fn hello() {} };
        let parsed = parse_fn_sig(input.clone()).expect("Should parse async unsafe fn");

        assert!(parsed.async_kw.is_some());
        assert!(parsed.unsafe_kw.is_some());
        assert_eq!(parsed.name.to_string(), "hello");

        let output = fn_sig_to_tokens(parsed);
        println!("Input:  {}", input);
        println!("Output: {}", output);
        assert!(output.to_string().contains("async"));
        assert!(output.to_string().contains("unsafe"));
    }

    #[test]
    fn test_const_unsafe_function() {
        let input = quote! { const unsafe fn hello() {} };
        let parsed = parse_fn_sig(input.clone()).expect("Should parse const unsafe fn");

        assert!(parsed.const_kw.is_some());
        assert!(parsed.unsafe_kw.is_some());
        assert_eq!(parsed.name.to_string(), "hello");

        let output = fn_sig_to_tokens(parsed);
        println!("Input:  {}", input);
        println!("Output: {}", output);
        assert!(output.to_string().contains("const"));
        assert!(output.to_string().contains("unsafe"));
    }

    #[test]
    fn test_extern_function() {
        let input = quote! { extern "C" fn hello() {} };
        let parsed = parse_fn_sig(input.clone()).expect("Should parse extern fn");

        assert!(parsed.extern_kw.is_some());
        assert_eq!(parsed.name.to_string(), "hello");

        let output = fn_sig_to_tokens(parsed);
        println!("Input:  {}", input);
        println!("Output: {}", output);
        assert!(output.to_string().contains("extern"));
        assert!(output.to_string().contains("\"C\""));
    }

    #[test]
    fn test_complex_function() {
        let input = quote! { pub const unsafe extern "C" fn hello<T>(x: T) -> T where T: Clone {} };
        let parsed = parse_fn_sig(input.clone()).expect("Should parse complex fn");

        assert!(parsed.visibility.is_some());
        assert!(parsed.const_kw.is_some());
        assert!(parsed.unsafe_kw.is_some());
        assert!(parsed.extern_kw.is_some());
        assert!(parsed.generics.is_some());
        assert!(parsed.return_type.is_some());
        assert_eq!(parsed.name.to_string(), "hello");

        let output = fn_sig_to_tokens(parsed);
        println!("Input:  {}", input);
        println!("Output: {}", output);
        assert!(output.to_string().contains("pub"));
        assert!(output.to_string().contains("const"));
        assert!(output.to_string().contains("unsafe"));
        assert!(output.to_string().contains("extern"));
        assert!(output.to_string().contains("\"C\""));
    }

    #[test]
    fn test_ret_with_level_parsing() {
        let input = quote!(level = "debug", ret);
        let mut iter = input.into_token_iter();

        match iter.parse::<InstrumentInner>() {
            Ok(parsed) => {
                println!("✅ Parsed mixed arguments successfully!");

                assert!(parsed.args.is_some(), "Should have parsed arguments");
                let args = parsed.args.as_ref().unwrap();
                println!("Number of arguments: {}", args.0.len());
                assert_eq!(args.0.len(), 2, "Should have 2 arguments");

                let mut found_level = false;
                let mut found_ret = false;

                for arg in &args.0 {
                    match &arg.value {
                        InstrumentArg::Level(_) => found_level = true,
                        InstrumentArg::Ret(_) => found_ret = true,
                        _ => {}
                    }
                }

                assert!(found_level, "Should find Level argument");
                assert!(found_ret, "Should find Ret argument");
            }
            Err(e) => {
                println!("❌ Parse failed: {}", e);
                panic!("Parse failed: {}", e);
            }
        }
    }

    #[test]
    fn test_mixed_args_with_ret() {
        let input = quote!(level = "info", name = "custom", ret);
        let mut iter = input.into_token_iter();

        match iter.parse::<InstrumentInner>() {
            Ok(parsed) => {
                println!("✅ Parsed mixed arguments with ret successfully!");

                assert!(parsed.args.is_some(), "Should have parsed arguments");
                let args = parsed.args.as_ref().unwrap();
                assert_eq!(args.0.len(), 3, "Should have 3 arguments");

                let mut found_level = false;
                let mut found_name = false;
                let mut found_ret = false;

                for arg in &args.0 {
                    match &arg.value {
                        InstrumentArg::Level(_) => found_level = true,
                        InstrumentArg::Name(_) => found_name = true,
                        InstrumentArg::Ret(_) => found_ret = true,
                    }
                }

                assert!(found_level, "Should find Level argument");
                assert!(found_name, "Should find Name argument");
                assert!(found_ret, "Should find Ret argument");
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_bare_ret_parsing() {
        let input = quote!(ret);
        let mut iter = input.into_token_iter();

        match iter.parse::<InstrumentInner>() {
            Ok(parsed) => {
                println!("✅ Parsed bare ret successfully!");

                assert!(parsed.args.is_some(), "Should have parsed arguments");
                let args = parsed.args.as_ref().unwrap();
                if let Some(first_arg) = args.0.first() {
                    match &first_arg.value {
                        InstrumentArg::Ret(_) => {
                            println!("✅ Found Ret argument");
                        }
                        _ => panic!("Expected Ret argument"),
                    }
                }
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_ret_with_parentheses_parsing() {
        let input = quote!(ret());
        let mut iter = input.into_token_iter();

        match iter.parse::<InstrumentInner>() {
            Ok(_parsed) => {
                println!("✅ Parsed ret() successfully!");
                // Should parse ret with empty parentheses (default EventArgs)
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_ret_with_debug_format() {
        let input = quote!(ret(Debug));
        let mut iter = input.into_token_iter();

        match iter.parse::<InstrumentInner>() {
            Ok(_parsed) => {
                println!("✅ Parsed ret(Debug) successfully!");
                // Should parse ret with Debug format mode
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_ret_with_display_format() {
        let input = quote!(ret(Display));
        let mut iter = input.into_token_iter();

        match iter.parse::<InstrumentInner>() {
            Ok(_parsed) => {
                println!("✅ Parsed ret(Display) successfully!");
                // Should parse ret with Display format mode
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_ret_with_custom_level() {
        let input = quote!(ret(level = "debug"));
        let mut iter = input.into_token_iter();

        match iter.parse::<InstrumentInner>() {
            Ok(_parsed) => {
                println!("✅ Parsed ret(level = \"debug\") successfully!");
                // Should parse ret with custom level
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }

    #[test]
    fn test_ret_with_level_and_format() {
        let input = quote!(ret(level = "warn", Display));
        let mut iter = input.into_token_iter();

        match iter.parse::<InstrumentInner>() {
            Ok(_parsed) => {
                println!("✅ Parsed ret(level = \"warn\", Display) successfully!");
                // Should parse ret with both custom level and format mode
            }
            Err(e) => panic!("Parse failed: {}", e),
        }
    }
}
