/// A code generator inspired by JavaPoet (https://github.com/square/javapoet)

mod imports;

use std::collections::BTreeSet;
use self::imports::ImportReceiver;

/// Build modifier lists.
#[macro_export]
macro_rules! mods {
    ($($modifier:expr),*) => {
        {
            let mut tmp_modifiers = Modifiers::new();

            $(
                tmp_modifiers.insert($modifier);
            )*

            tmp_modifiers
        }
    }
}

/// Tool to build statements.
#[macro_export]
macro_rules! stmt {
    ($($var:expr),*) => {{
        let mut statement = Statement::new();
        $(statement.push($var);)*
        statement
    }};
}

macro_rules! as_converter {
    ($as_type:ident, $type:ty, $method:ident) => {
        pub trait $as_type {
            fn $method(self) -> $type;
        }

        impl<'a, A> $as_type for &'a A
            where A: $as_type + Clone
        {
            fn $method(self) -> $type {
                self.clone().$method()
            }
        }

        impl $as_type for $type {
            fn $method(self) -> $type {
                self
            }
        }
    }
}

fn java_quote_string(input: &str) -> String {
    let mut out = String::new();
    let mut it = input.chars();

    out.push('"');

    while let Some(c) = it.next() {
        match c {
            '\t' => out.push_str("\\t"),
            '\u{0007}' => out.push_str("\\b"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{0014}' => out.push_str("\\f"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }

    out.push('"');
    out
}

impl ImportReceiver for BTreeSet<ClassType> {
    fn receive(&mut self, ty: &ClassType) {
        self.insert(ty.clone());
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Modifier {
    Public,
    Protected,
    Private,
    Static,
    Final,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Modifiers {
    pub modifiers: BTreeSet<Modifier>,
}

impl Modifiers {
    pub fn new() -> Modifiers {
        Modifiers { modifiers: BTreeSet::new() }
    }

    pub fn insert(&mut self, modifier: Modifier) {
        self.modifiers.insert(modifier);
    }

    pub fn format(&self) -> String {
        let mut out: Vec<String> = Vec::new();

        for m in &self.modifiers {
            out.push(match *m {
                Modifier::Public => "public".to_owned(),
                Modifier::Protected => "protected".to_owned(),
                Modifier::Private => "private".to_owned(),
                Modifier::Static => "static".to_owned(),
                Modifier::Final => "final".to_owned(),
            });
        }

        out.join(" ")
    }

    pub fn is_empty(&self) -> bool {
        self.modifiers.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum Section {
    Block(Block),
    Statement(Statement),
    Literal(Vec<String>),
    Spacing,
}

as_converter!(AsSection, Section, as_section);

impl AsSection for Block {
    fn as_section(self) -> Section {
        Section::Block(self)
    }
}

impl AsSection for Statement {
    fn as_section(self) -> Section {
        Section::Statement(self)
    }
}

impl AsSection for Vec<String> {
    fn as_section(self) -> Section {
        Section::Literal(self)
    }
}

#[derive(Debug, Clone)]
pub enum Variable {
    Literal(String),
    Type(Type),
    String(String),
    Statement(Statement),
    Spacing,
}

as_converter!(AsVariable, Variable, as_variable);

impl AsVariable for &'static str {
    fn as_variable(self) -> Variable {
        Variable::Literal(self.to_owned())
    }
}

impl AsVariable for String {
    fn as_variable(self) -> Variable {
        Variable::Literal(self)
    }
}

impl AsVariable for Type {
    fn as_variable(self) -> Variable {
        Variable::Type(self)
    }
}

impl AsVariable for ClassType {
    fn as_variable(self) -> Variable {
        Variable::Type(self.as_type())
    }
}

impl AsVariable for Statement {
    fn as_variable(self) -> Variable {
        Variable::Statement(self)
    }
}

impl AsVariable for FieldSpec {
    fn as_variable(self) -> Variable {
        Variable::Literal(self.name)
    }
}

impl AsVariable for ArgumentSpec {
    fn as_variable(self) -> Variable {
        Variable::Literal(self.name)
    }
}

#[derive(Debug, Clone)]
pub struct Statement {
    parts: Vec<Variable>,
}

impl Statement {
    pub fn new() -> Statement {
        Statement { parts: Vec::new() }
    }

    pub fn push<V>(&mut self, variable: V)
        where V: AsVariable
    {
        self.parts.push(variable.as_variable());
    }

    pub fn push_arguments<S, A>(&mut self, arguments: &Vec<S>, separator: A)
        where S: AsStatement + Clone,
              A: AsVariable + Clone
    {
        if arguments.is_empty() {
            return;
        }

        let mut out: Statement = Statement::new();

        for a in arguments {
            out.push(a.as_statement());
        }

        self.push(out.join(separator));
    }

    pub fn join<A>(self, separator: A) -> Statement
        where A: AsVariable + Clone
    {
        let mut it = self.parts.into_iter();

        let part = match it.next() {
            Some(part) => part,
            None => return Statement::new(),
        };

        let mut parts: Vec<Variable> = Vec::new();
        parts.push(part);

        let sep = &separator;

        while let Some(part) = it.next() {
            parts.push(sep.as_variable());
            parts.push(part);
        }

        Statement { parts: parts }
    }

    pub fn format(&self, level: usize) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut current: Vec<String> = Vec::new();

        for part in &self.parts {
            match *part {
                Variable::Type(ref ty) => {
                    current.push(ty.format(level));
                }
                Variable::String(ref string) => {
                    current.push(java_quote_string(string));
                }
                Variable::Statement(ref stmt) => {
                    current.push(stmt.format(level).join(" "));
                }
                Variable::Literal(ref content) => {
                    current.push(content.to_owned());
                }
                Variable::Spacing => {
                    out.push(current.join(""));
                    current.clear();
                }
            }
        }

        if !current.is_empty() {
            out.push(current.join(""));
            current.clear();
        }

        out
    }
}

as_converter!(AsStatement, Statement, as_statement);

impl AsStatement for FieldSpec {
    fn as_statement(self) -> Statement {
        let mut s = Statement::new();

        if !self.modifiers.is_empty() {
            s.push(stmt![self.modifiers.format(), " "]);
        }

        s.push(stmt![self.ty, " ", self.name]);

        s
    }
}

impl AsStatement for AnnotationSpec {
    fn as_statement(self) -> Statement {
        let mut s = Statement::new();
        s.push(stmt!["@", self.ty]);

        if !self.arguments.is_empty() {
            s.push("(");
            s.push_arguments(&self.arguments, ", ");
            s.push(")");
        }

        s
    }
}

impl AsStatement for ArgumentSpec {
    fn as_statement(self) -> Statement {
        let mut s = Statement::new();

        for a in &self.annotations {
            s.push(a.as_statement());
            s.push(Variable::Spacing);
        }

        if !self.modifiers.is_empty() {
            s.push(stmt!(self.modifiers.format(), " "));
        }

        s.push(stmt![self.ty, " ", self.name]);

        s
    }
}

#[derive(Debug, Clone)]
pub struct Sections {
    sections: Vec<Section>,
}

impl Sections {
    pub fn new() -> Sections {
        Sections { sections: Vec::new() }
    }

    pub fn push<S>(&mut self, section: S)
        where S: AsSection
    {
        self.sections.push(section.as_section());
    }

    pub fn extend(&mut self, sections: &Sections) {
        self.sections.extend(sections.sections.iter().map(Clone::clone));
    }

    pub fn format(&self, level: usize, current: &str, indent: &str) -> Vec<String> {
        let mut out = Vec::new();

        for section in &self.sections {
            match *section {
                Section::Statement(ref statement) => {
                    for line in statement.format(level) {
                        out.push(format!("{}{};", current, line));
                    }
                }
                Section::Block(ref block) => {
                    out.extend(block.format(level, current, indent));
                }
                Section::Spacing => {
                    out.push("".to_owned());
                }
                Section::Literal(ref content) => {
                    for line in content {
                        out.push(format!("{}{}", current, line));
                    }
                }
            }
        }

        out
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    open: Option<Statement>,
    close: Option<Statement>,
    sections: Sections,
}

impl Block {
    pub fn new() -> Block {
        Block {
            open: None,
            close: None,
            sections: Sections::new(),
        }
    }

    pub fn open<S>(&mut self, open: S)
        where S: AsStatement
    {
        self.open = Some(open.as_statement())
    }

    pub fn close<S>(&mut self, close: S)
        where S: AsStatement
    {
        self.close = Some(close.as_statement())
    }

    pub fn push<S>(&mut self, section: S)
        where S: AsSection
    {
        self.sections.push(section);
    }

    pub fn extend(&mut self, sections: &Sections) {
        self.sections.extend(sections);
    }

    pub fn format(&self, level: usize, current: &str, indent: &str) -> Vec<String> {
        let mut out = Vec::new();

        if let Some(ref open) = self.open {
            let mut it = open.format(level).into_iter().peekable();

            while let Some(line) = it.next() {
                if it.peek().is_none() {
                    out.push(format!("{}{} {{", current, line).to_owned());
                } else {
                    out.push(format!("{}{}", current, line).to_owned());
                }
            }
        } else {
            out.push(format!("{}{{", current).to_owned());
        }

        out.extend(self.sections.format(level, &format!("{}{}", current, indent), indent));

        if let Some(ref close) = self.close {
            let close = close.format(level).join(" ");
            out.push(format!("{}}} {};", current, close).to_owned());
        } else {
            out.push(format!("{}}}", current).to_owned());
        }

        out
    }
}

/// Complete types, including generic arguments.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ClassType {
    pub package: String,
    pub name: String,
    pub arguments: Vec<Type>,
}

impl ClassType {
    pub fn new(package: &str, name: &str, arguments: Vec<Type>) -> ClassType {
        ClassType {
            package: package.to_owned(),
            name: name.to_owned(),
            arguments: arguments,
        }
    }

    pub fn with_arguments<A>(&self, arguments: Vec<A>) -> ClassType
        where A: AsType
    {
        let arguments = arguments.into_iter().map(AsType::as_type).collect();
        ClassType::new(&self.package, &self.name, arguments)
    }

    pub fn to_raw(&self) -> ClassType {
        ClassType::new(&self.package, &self.name, vec![])
    }

    pub fn format(&self, level: usize) -> String {
        let mut out = String::new();

        out.push_str(&self.name);

        if !self.arguments.is_empty() {
            let mut arguments = Vec::new();

            let level = level + 1;

            for g in &self.arguments {
                arguments.push(g.format(level));
            }

            let joined = arguments.join(", ");

            out.push('<');
            out.push_str(&joined);
            out.push('>');
        }

        out
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct PrimitiveType {
    pub primitive: String,
    pub boxed: String,
}

impl PrimitiveType {
    pub fn new(primitive: &str, boxed: &str) -> PrimitiveType {
        PrimitiveType {
            primitive: primitive.to_owned(),
            boxed: boxed.to_owned(),
        }
    }

    pub fn format(&self, level: usize) -> String {
        if level <= 0 {
            self.primitive.clone()
        } else {
            self.boxed.clone()
        }
    }
}

/// Raw (importable) types.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum Type {
    Primitive(PrimitiveType),
    Class(ClassType),
}

impl Type {
    pub fn primitive(primitive: &str, boxed: &str) -> PrimitiveType {
        PrimitiveType::new(primitive, boxed)
    }

    pub fn class(package: &str, name: &str) -> ClassType {
        ClassType::new(package, name, vec![])
    }

    pub fn format(&self, level: usize) -> String {
        match *self {
            Type::Primitive(ref primitive) => primitive.format(level),
            Type::Class(ref class) => class.format(level),
        }
    }
}

as_converter!(AsType, Type, as_type);

/// Implementation for ClassType to Type conversion.
impl AsType for ClassType {
    fn as_type(self) -> Type {
        Type::Class(self)
    }
}

/// Implementation for PrimitiveType to Type conversion.
impl AsType for PrimitiveType {
    fn as_type(self) -> Type {
        Type::Primitive(self)
    }
}

#[derive(Debug, Clone)]
pub struct MethodArgument {
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub modifiers: Modifiers,
    pub ty: Type,
    pub name: String,
}

impl FieldSpec {
    pub fn new<I>(modifiers: Modifiers, ty: I, name: &str) -> FieldSpec
        where I: AsType
    {
        FieldSpec {
            modifiers: modifiers,
            ty: ty.as_type(),
            name: name.to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstructorSpec {
    pub modifiers: Modifiers,
    pub annotations: Vec<AnnotationSpec>,
    pub arguments: Vec<ArgumentSpec>,
    pub sections: Sections,
}

impl ConstructorSpec {
    pub fn new(modifiers: Modifiers) -> ConstructorSpec {
        ConstructorSpec {
            modifiers: modifiers,
            annotations: Vec::new(),
            arguments: Vec::new(),
            sections: Sections::new(),
        }
    }

    pub fn push_annotation<A>(&mut self, annotation: A)
        where A: AsAnnotationSpec
    {
        self.annotations.push(annotation.as_annotation_spec());
    }

    pub fn push_argument<A>(&mut self, argument: A)
        where A: AsArgumentSpec
    {
        self.arguments.push(argument.as_argument_spec());
    }

    pub fn push<S>(&mut self, section: S)
        where S: AsSection
    {
        self.sections.push(section);
    }

    pub fn as_block(&self, enclosing: &str) -> Block {
        let mut open = Statement::new();

        for a in &self.annotations {
            open.push(a.as_statement());
            open.push(Variable::Spacing);
        }

        if !self.modifiers.is_empty() {
            open.push(stmt![self.modifiers.format(), " "]);
        }

        open.push(stmt![enclosing.to_owned(), "("]);
        open.push_arguments(&self.arguments, ", ");
        open.push(stmt![")"]);

        let mut block = Block::new();
        block.open(open);
        block.extend(&self.sections);

        block
    }
}

#[derive(Debug, Clone)]
pub struct AnnotationSpec {
    pub ty: Type,
    pub arguments: Vec<Statement>,
}

impl AnnotationSpec {
    pub fn new<I>(ty: I) -> AnnotationSpec
        where I: AsType
    {
        AnnotationSpec {
            ty: ty.as_type(),
            arguments: Vec::new(),
        }
    }

    pub fn push_argument<S>(&mut self, statement: S)
        where S: AsStatement
    {
        self.arguments.push(statement.as_statement());
    }
}

as_converter!(AsAnnotationSpec, AnnotationSpec, as_annotation_spec);

#[derive(Debug, Clone)]
pub struct ArgumentSpec {
    pub modifiers: Modifiers,
    pub ty: Type,
    pub name: String,
    pub annotations: Vec<AnnotationSpec>,
}

impl ArgumentSpec {
    pub fn new<I>(modifiers: Modifiers, ty: I, name: &str) -> ArgumentSpec
        where I: AsType
    {
        ArgumentSpec {
            modifiers: modifiers,
            ty: ty.as_type(),
            name: name.to_owned(),
            annotations: Vec::new(),
        }
    }

    pub fn push_annotation(&mut self, annotation: &AnnotationSpec) {
        self.annotations.push(annotation.clone());
    }
}

as_converter!(AsArgumentSpec, ArgumentSpec, as_argument_spec);

#[derive(Debug, Clone)]
pub struct MethodSpec {
    pub modifiers: Modifiers,
    pub name: String,
    pub annotations: Vec<AnnotationSpec>,
    pub arguments: Vec<ArgumentSpec>,
    pub returns: Option<Type>,
    pub sections: Sections,
}

impl MethodSpec {
    pub fn new(modifiers: Modifiers, name: &str) -> MethodSpec {
        MethodSpec {
            modifiers: modifiers,
            name: name.to_owned(),
            annotations: Vec::new(),
            arguments: Vec::new(),
            returns: None,
            sections: Sections::new(),
        }
    }

    pub fn push_annotation(&mut self, annotation: &AnnotationSpec) {
        self.annotations.push(annotation.clone());
    }

    pub fn push_argument<A>(&mut self, argument: A)
        where A: AsArgumentSpec
    {
        self.arguments.push(argument.as_argument_spec());
    }

    pub fn returns<I>(&mut self, returns: I)
        where I: AsType
    {
        self.returns = Some(returns.as_type())
    }

    pub fn push<S>(&mut self, section: S)
        where S: AsSection
    {
        self.sections.push(section);
    }

    pub fn as_block(&self) -> Block {
        let mut open = Statement::new();

        for a in &self.annotations {
            open.push(a.as_statement());
            open.push(Variable::Spacing);
        }

        if !self.modifiers.is_empty() {
            open.push(stmt!(self.modifiers.format(), " "));
        }

        match self.returns {
            None => open.push("void "),
            Some(ref returns) => open.push(stmt![returns, " "]),
        }

        open.push(stmt![&self.name, "("]);

        if !self.arguments.is_empty() {
            let mut arguments: Statement = Statement::new();

            for a in &self.arguments {
                arguments.push(a.as_statement());
            }

            open.push(arguments.join(", "));
        }

        open.push(stmt![")"]);

        let mut block = Block::new();
        block.open(open);
        block.extend(&self.sections);

        block
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceSpec {
    pub modifiers: Modifiers,
    pub name: String,
    pub annotations: Vec<AnnotationSpec>,
    pub elements: Vec<ElementSpec>,
}

impl InterfaceSpec {
    pub fn new(modifiers: Modifiers, name: &str) -> InterfaceSpec {
        InterfaceSpec {
            modifiers: modifiers,
            name: name.to_owned(),
            annotations: Vec::new(),
            elements: Vec::new(),
        }
    }

    pub fn push_annotation(&mut self, annotation: &AnnotationSpec) {
        self.annotations.push(annotation.clone());
    }

    pub fn push_class(&mut self, class: &ClassSpec) {
        self.elements.push(ElementSpec::Class(class.clone()))
    }

    pub fn push_interface(&mut self, interface: &InterfaceSpec) {
        self.elements.push(ElementSpec::Interface(interface.clone()))
    }

    pub fn push_statement(&mut self, statement: &Statement) {
        self.elements.push(ElementSpec::Statement(statement.clone()))
    }

    pub fn push_literal(&mut self, content: &Vec<String>) {
        self.elements.push(ElementSpec::Literal(content.clone()))
    }

    pub fn as_block(&self) -> Block {
        let mut open = Statement::new();

        for a in &self.annotations {
            open.push(a.as_statement());
            open.push(Variable::Spacing);
        }

        if !self.modifiers.is_empty() {
            open.push(stmt!(self.modifiers.format(), " "));
        }

        open.push(stmt!["interface ", &self.name]);

        let mut block = Block::new();
        block.open(open);

        let mut first: bool = true;

        for element in &self.elements {
            if first {
                first = false;
            } else {
                block.push(Section::Spacing);
            }

            element.add_to_block(&mut block);
        }

        block
    }
}

#[derive(Debug, Clone)]
pub struct ClassSpec {
    pub modifiers: Modifiers,
    pub name: String,
    pub annotations: Vec<AnnotationSpec>,
    pub fields: Vec<FieldSpec>,
    pub constructors: Vec<ConstructorSpec>,
    pub methods: Vec<MethodSpec>,
    pub elements: Vec<ElementSpec>,
}

impl ClassSpec {
    pub fn new(modifiers: Modifiers, name: &str) -> ClassSpec {
        ClassSpec {
            modifiers: modifiers,
            name: name.to_owned(),
            annotations: Vec::new(),
            fields: Vec::new(),
            constructors: Vec::new(),
            methods: Vec::new(),
            elements: Vec::new(),
        }
    }

    pub fn push_annotation(&mut self, annotation: &AnnotationSpec) {
        self.annotations.push(annotation.clone());
    }

    pub fn push_field(&mut self, field: &FieldSpec) {
        self.fields.push(field.clone());
    }

    pub fn push_constructor(&mut self, constructor: &ConstructorSpec) {
        self.constructors.push(constructor.clone());
    }

    pub fn push_method(&mut self, method: &MethodSpec) {
        self.methods.push(method.clone());
    }

    pub fn push_class(&mut self, class: &ClassSpec) {
        self.elements.push(ElementSpec::Class(class.clone()))
    }

    pub fn push_interface(&mut self, interface: &InterfaceSpec) {
        self.elements.push(ElementSpec::Interface(interface.clone()))
    }

    pub fn push_statement(&mut self, statement: &Statement) {
        self.elements.push(ElementSpec::Statement(statement.clone()))
    }

    pub fn push_literal(&mut self, content: &Vec<String>) {
        self.elements.push(ElementSpec::Literal(content.clone()))
    }

    pub fn as_block(&self) -> Block {
        let mut open = Statement::new();

        for a in &self.annotations {
            open.push(a.as_statement());
            open.push(Variable::Spacing);
        }

        if !self.modifiers.is_empty() {
            open.push(stmt![self.modifiers.format(), " "]);
        }

        open.push(stmt!["class ", &self.name]);

        let mut block = Block::new();
        block.open(open);

        for field in &self.fields {
            block.push(field.as_statement());
        }

        /// TODO: figure out a better way...
        let mut first = self.fields.is_empty();

        for constructor in &self.constructors {
            if first {
                first = false;
            } else {
                block.push(Section::Spacing);
            }

            block.push(constructor.as_block(&self.name));
        }

        for method in &self.methods {
            if first {
                first = false;
            } else {
                block.push(Section::Spacing);
            }

            block.push(method.as_block());
        }

        for element in &self.elements {
            if first {
                first = false;
            } else {
                block.push(Section::Spacing);
            }

            element.add_to_block(&mut block);
        }

        block
    }
}

#[derive(Debug, Clone)]
pub enum ElementSpec {
    Class(ClassSpec),
    Interface(InterfaceSpec),
    Statement(Statement),
    Literal(Vec<String>),
}

impl ElementSpec {
    pub fn add_to_block(&self, target: &mut Block) {
        match *self {
            ElementSpec::Class(ref class) => {
                target.push(class.as_block());
            }
            ElementSpec::Interface(ref interface) => {
                target.push(interface.as_block());
            }
            ElementSpec::Statement(ref statement) => {
                target.push(statement);
            }
            ElementSpec::Literal(ref content) => {
                target.push(content);
            }
        };
    }

    pub fn add_to_sections(&self, target: &mut Sections) {
        match *self {
            ElementSpec::Class(ref class) => {
                target.push(class.as_block());
            }
            ElementSpec::Interface(ref interface) => {
                target.push(interface.as_block());
            }
            ElementSpec::Statement(ref statement) => {
                target.push(statement);
            }
            ElementSpec::Literal(ref content) => {
                target.push(content);
            }
        };
    }
}

#[derive(Debug, Clone)]
pub struct FileSpec {
    pub package: String,
    pub elements: Vec<ElementSpec>,
}

impl FileSpec {
    pub fn new(package: &str) -> FileSpec {
        FileSpec {
            package: package.to_owned(),
            elements: Vec::new(),
        }
    }

    pub fn push_class(&mut self, class: &ClassSpec) {
        self.elements.push(ElementSpec::Class(class.clone()))
    }

    pub fn push_interface(&mut self, interface: &InterfaceSpec) {
        self.elements.push(ElementSpec::Interface(interface.clone()))
    }

    pub fn format(&self) -> String {
        let mut sections = Sections::new();

        sections.push(&stmt!["package ", &self.package]);
        sections.push(Section::Spacing);

        let mut receiver: BTreeSet<ClassType> = BTreeSet::new();

        receiver.import_all(&self.elements);

        let imports: BTreeSet<ClassType> = receiver.into_iter()
            .filter(|t| t.package != "java.lang")
            .filter(|t| t.package != self.package)
            .map(|t| t.to_raw())
            .collect();

        if !imports.is_empty() {
            for t in imports {
                sections.push(&stmt!["import ", t.package, ".", t.name]);
            }

            sections.push(Section::Spacing);
        }

        for element in &self.elements {
            element.add_to_sections(&mut sections);
        }

        let mut out = String::new();

        for line in sections.format(0usize, "", "  ") {
            out.push_str(&line);
            out.push('\n');
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_java() {
        let string_type = Type::class("java.lang", "String");
        let list_type = Type::class("java.util", "List");
        let json_creator_type = Type::class("com.fasterxml.jackson.annotation", "JsonCreator");
        let list_of_strings = list_type.with_arguments(vec![&string_type]);

        let values_field = FieldSpec::new(mods![Modifier::Private, Modifier::Final],
                                          &list_of_strings,
                                          "values");

        let values_argument = ArgumentSpec::new(mods![Modifier::Final], &list_of_strings, "values");

        let mut constructor = ConstructorSpec::new(mods![Modifier::Public]);
        constructor.push_annotation(AnnotationSpec::new(json_creator_type));
        constructor.push_argument(&values_argument);
        constructor.push(stmt!["this.values = ", values_argument]);

        let mut values_getter = MethodSpec::new(mods![Modifier::Public], "getValues");
        values_getter.returns(&list_of_strings);
        values_getter.push(stmt!["return this.", &values_field]);

        let mut class = ClassSpec::new(mods![Modifier::Public], "Test");
        class.push_field(&values_field);
        class.push_constructor(&constructor);
        class.push_method(&values_getter);

        let mut file = FileSpec::new("se.tedro");
        file.push_class(&class);

        let result = file.format();

        let reference = ::std::str::from_utf8(include_bytes!("tests/Test.java")).unwrap();
        assert_eq!(reference, result);
    }
}