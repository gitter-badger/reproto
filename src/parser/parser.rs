#![allow(unconditional_recursion)]

use pest::prelude::*;
use std::collections::LinkedList;
use super::ast::*;
use super::errors::*;

/// Check if character is an indentation character.
fn is_indent(c: char) -> bool {
    match c {
        ' ' | '\t' => true,
        _ => false,
    }
}

/// Find the number of whitespace characters that the given string is indented.
fn find_indent(input: &str) -> Option<usize> {
    input.chars().enumerate().find(|&(_, c)| !is_indent(c)).map(|(i, _)| i)
}

fn code_block_indent(input: &str) -> Option<(usize, usize, usize)> {
    let mut indent: Option<usize> = None;

    let mut start = 0;
    let mut end = 0;

    let mut first_line = false;

    for (line_no, line) in input.lines().enumerate() {
        if let Some(current) = find_indent(line) {
            end = line_no + 1;

            if indent.map(|i| i > current).unwrap_or(true) {
                indent = Some(current);
            }

            first_line = true;
        } else {
            if !first_line {
                start += 1;
            }
        }
    }

    indent.map(|indent| (indent, start, end - start))
}

/// Strip common indent from all input lines.
fn strip_code_block(input: &str) -> Vec<String> {
    if let Some((indent, empty_start, len)) = code_block_indent(input) {
        input.lines()
            .skip(empty_start)
            .take(len)
            .map(|line| {
                if line.len() < indent {
                    line.to_owned()
                } else {
                    (&line[indent..]).to_owned()
                }
            })
            .collect()
    } else {
        input.lines().map(ToOwned::to_owned).collect()
    }
}

/// Decode an escaped string.
fn decode_escaped_string(input: &str) -> Result<String> {
    let mut out = String::new();
    let mut it = input.chars().skip(1).peekable();

    loop {
        let c = match it.next() {
            None => break,
            Some(c) => c,
        };

        // strip end quote
        if it.peek().is_none() {
            break;
        }

        if c == '\\' {
            let escaped = match it.next().ok_or("expected character")? {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'u' => decode_unicode4(&mut it)?,
                _ => return Err(ErrorKind::InvalidEscape.into()),
            };

            out.push(escaped);
            continue;
        }

        out.push(c);
    }

    Ok(out)
}

/// Decode the next four characters as a unicode escape sequence.
fn decode_unicode4(it: &mut Iterator<Item = char>) -> Result<char> {
    let mut res = 0u32;

    for x in 0..4u32 {
        let c = it.next().ok_or("expected hex character")?.to_string();
        let c = u32::from_str_radix(&c, 16)?;
        res += c << (4 * (3 - x));
    }

    Ok(::std::char::from_u32(res).ok_or("expected valid character")?)
}

impl_rdp! {
    grammar! {
        file = _{ package_decl ~ use_decl* ~ decl* ~ eoi }
        decl = { type_decl | interface_decl | tuple_decl | enum_decl }

        use_decl = { use_keyword ~ package_ident ~ use_as? ~ semi_colon }
        use_as = { as_keyword ~ identifier }

        package_decl = { package_keyword ~ package_ident ~ semi_colon }

        type_decl = { type_keyword ~ type_identifier ~ left_curly ~ type_body ~ right_curly }
        type_body = _{ member* }

        tuple_decl = { tuple_keyword ~ type_identifier ~ left_curly ~ tuple_body ~ right_curly }
        tuple_body = _{ member* }

        interface_decl = { interface_keyword ~ type_identifier ~ left_curly ~ interface_body ~ right_curly }
        interface_body = _{ member* ~ sub_type* }

        enum_decl = { enum_keyword ~ type_identifier ~ left_curly ~ enum_body ~ right_curly }
        enum_body = _{ enum_body_value* ~ member* }
        enum_body_value = { enum_value }

        sub_type = { type_identifier ~ left_curly ~ sub_type_body ~ right_curly }
        sub_type_body = _{ member* }

        member = { option_decl | match_decl | field | code_block }
        field = { identifier ~ optional? ~ colon ~ type_spec ~ field_as? ~ semi_colon }
        field_as = { as_keyword ~ value }
        code_block = @{ identifier ~ whitespace* ~ code_start ~ code_body ~ code_end }
        code_body = { (!(["}}"]) ~ any)* }

        enum_value = { enum_name ~ enum_arguments? ~ enum_ordinal? ~ semi_colon }
        enum_name = { type_identifier }
        enum_arguments = { (left_paren ~ (value ~ (comma ~ value)*) ~ right_paren) }
        enum_ordinal = { equals ~ value }
        option_decl = { identifier ~ (value ~ (comma ~ value)*) ~ semi_colon }

        match_decl = { match_keyword ~ left_curly ~ match_member_entry* ~ right_curly }
        match_member_entry = { match_member }
        match_member = { match_condition ~ hash_rocket ~ value ~ semi_colon }
        match_condition = { match_variable | match_value }
        match_variable = { identifier ~ colon ~ type_spec }
        match_value = { value }

        package_ident = @{ identifier ~ (dot ~ identifier)* }

        type_spec = _{
            float_type |
            double_type |
            signed_type |
            unsigned_type |
            boolean_type |
            string_type |
            bytes_type |
            any_type |
            map_type |
            array_type |
            custom_type
        }

        // Types
        float_type = @{ ["float"] }
        double_type = @{ ["double"] }
        signed_type = @{ ["signed"] ~ type_bits? }
        unsigned_type = @{ ["unsigned"] ~ type_bits? }
        boolean_type = @{ ["boolean"] }
        string_type = @{ ["string"] }
        bytes_type = @{ ["bytes"] }
        any_type = @{ ["any"] }
        map_type = { left_curly ~ type_spec ~ colon ~ type_spec ~ right_curly }
        array_type = { bracket_start ~ type_spec ~ bracket_end }
        custom_type = @{ used_prefix? ~ type_identifier ~ (dot ~ type_identifier)* }

        used_prefix = @{ identifier ~ scope }

        // Keywords and tokens
        enum_keyword = @{ ["enum"] }
        use_keyword = @{ ["use"] }
        as_keyword = @{ ["as"] }
        package_keyword = @{ ["package"] }
        type_keyword = @{ ["type"] }
        tuple_keyword = @{ ["tuple"] }
        interface_keyword = @{ ["interface"] }
        match_keyword = @{ ["match"] }
        hash_rocket = @{ ["=>"] }
        comma = @{ [","] }
        colon = @{ [":"] }
        scope = @{ ["::"] }
        semi_colon = @{ [";"] }
        left_curly = @{ ["{"] }
        right_curly = @{ ["}"] }
        bracket_start = @{ ["["] }
        bracket_end = @{ ["]"] }
        code_start = @{ ["{{"] }
        code_end = @{ ["}}"] }
        left_paren = @{ ["("] }
        right_paren = @{ [")"] }
        forward_slash = @{ ["/"] }
        optional = @{ ["?"] }
        equals = @{ ["="] }
        dot = @{ ["."] }

        type_bits = _{ (forward_slash ~ unsigned) }

        optional_value_list = { value ~ (comma ~ value)* }
        value = { instance | constant | array | boolean | identifier | string | number }

        instance = { custom_type ~ instance_arguments }
        instance_arguments = { (left_paren ~ (field_init ~ (comma ~ field_init)*)? ~ right_paren) }

        constant = { custom_type }

        array = { bracket_start ~ optional_value_list? ~ bracket_end }

        field_init = { field_name ~ colon ~ value }
        field_name = { identifier }

        identifier = @{ ['a'..'z'] ~ (['0'..'9'] | ['a'..'z'] | ["_"])* }
        type_identifier = @{ ['A'..'Z'] ~ (['A'..'Z'] | ['a'..'z'] | ['0'..'9'])* }

        string  = @{ ["\""] ~ (escape | !(["\""] | ["\\"]) ~ any)* ~ ["\""] }
        escape  =  _{ ["\\"] ~ (["\""] | ["\\"] | ["/"] | ["n"] | ["r"] | ["t"] | unicode) }
        unicode =  _{ ["u"] ~ hex ~ hex ~ hex ~ hex }
        hex     =  _{ ['0'..'9'] | ['a'..'f'] }

        unsigned = @{ int }
        number   = @{ ["-"]? ~ int ~ (["."] ~ ['0'..'9']+)? ~ (["e"] ~ int)? }
        int      =  _{ ["0"] | ['1'..'9'] ~ ['0'..'9']* }

        boolean = { ["true"] | ["false"] }

        whitespace = _{ [" "] | ["\t"] | ["\r"] | ["\n"] }

        comment = _{
            // line comment
            ( ["//"] ~ (!(["\r"] | ["\n"]) ~ any)* ~ (["\n"] | ["\r\n"] | ["\r"] | eoi) ) |
            // block comment
            ( ["/*"] ~ (!(["*/"]) ~ any)* ~ ["*/"] )
        }
    }

    process! {
        _file(&self) -> Result<File> {
            (
                _: package_decl,
                _: package_keyword,
                package: _package(), _: semi_colon,
                uses: _use_list(),
                decls: _decl_list(),
            ) => {
                let package = package;
                let uses = uses?.into_iter().collect();
                let decls = decls?.into_iter().collect();

                Ok(File {
                    package: package,
                    uses: uses,
                    decls: decls
                })
            },
        }

        _use_list(&self) -> Result<LinkedList<AstLoc<UseDecl>>> {
            (token: use_decl, use_decl: _use_decl(), tail: _use_list()) => {
                let pos = (token.start, token.end);
                let mut tail = tail?;
                tail.push_front(AstLoc::new(use_decl, pos));
                Ok(tail)
            },

            () => Ok(LinkedList::new()),
        }

        _use_decl(&self) -> UseDecl {
            (_: use_keyword, package: _package(), alias: _use_as(), _: semi_colon) => {
                UseDecl {
                    package: package,
                    alias: alias,
                }
            }
        }

        _use_as(&self) -> Option<String> {
            (_: use_as, _: as_keyword, &alias: identifier) => Some(alias.to_owned()),
            () => None,
        }

        _package(&self) -> AstLoc<RpPackage> {
            (token: package_ident, idents: _ident_list()) => {
                let pos = (token.start, token.end);
                let idents = idents;
                let package = RpPackage::new(idents.into_iter().collect());
                AstLoc::new(package, pos)
            },
        }

        _decl_list(&self) -> Result<LinkedList<AstLoc<Decl>>> {
            (token: decl, value: _decl(), tail: _decl_list()) => {
                let mut tail = tail?;
                let pos = (token.start, token.end);
                tail.push_front(AstLoc::new(value?, pos));
                Ok(tail)
            },

            () => Ok(LinkedList::new()),
        }

        _decl(&self) -> Result<Decl> {
            (
                _: type_decl,
                _: type_keyword,
                &name: type_identifier,
                _: left_curly,
                members: _member_list(),
                _: right_curly,
            ) => {
                let members = members?.into_iter().collect();

                let body = TypeBody {
                    name: name.to_owned(),
                    members: members
                };

                Ok(Decl::Type(body))
            },

            (
                _: tuple_decl,
                _: tuple_keyword,
                &name: type_identifier,
                _: left_curly,
                members: _member_list(),
                _: right_curly,
            ) => {
                let members = members?.into_iter().collect();

                let body = TupleBody {
                    name: name.to_owned(),
                    members: members,
                };

                Ok(Decl::Tuple(body))
            },

            (
                _: interface_decl,
                _: interface_keyword,
                &name: type_identifier,
                _: left_curly,
                members: _member_list(),
                sub_types: _sub_type_list(),
                _: right_curly,
            ) => {
                let members = members?.into_iter().collect();
                let sub_types = sub_types?.into_iter().collect();

                let body = InterfaceBody {
                    name: name.to_owned(),
                    members: members,
                    sub_types: sub_types,
                };

                Ok(Decl::Interface(body))
            },

            (
                _: enum_decl,
                _: enum_keyword,
                &name: type_identifier,
                _: left_curly,
                values: _enum_value_list(),
                members: _member_list(),
                _: right_curly,
            ) => {
                let values = values?.into_iter().collect();
                let members = members?.into_iter().collect();

                let body = EnumBody {
                    name: name.to_owned(),
                    values: values,
                    members: members,
                };

                Ok(Decl::Enum(body))
            },
        }

        _enum_value_list(&self) -> Result<LinkedList<AstLoc<EnumValue>>> {
            (_: enum_body_value, value: _enum_value(), tail: _enum_value_list()) => {
                let mut tail = tail?;
                tail.push_front(value?);
                Ok(tail)
            },

            () => Ok(LinkedList::new()),
        }

        _enum_value(&self) -> Result<AstLoc<EnumValue>> {
            (
                token: enum_value,
                name_token: enum_name,
                &name: type_identifier,
                values: _enum_arguments(),
                ordinal: _enum_ordinal(),
                _: semi_colon
             ) => {
                let name = AstLoc::new(name.to_owned(), (name_token.start, name_token.end));

                let enum_value = EnumValue {
                    name: name,
                    arguments: values?.into_iter().collect(),
                    ordinal: ordinal?
                };

                Ok(AstLoc::new(enum_value, (token.start, token.end)))
            },
        }

        _enum_arguments(&self) -> Result<LinkedList<AstLoc<RpValue>>> {
            (_: enum_arguments, _: left_paren, values: _value_list(), _: right_paren) => values,
            () => Ok(LinkedList::new()),
        }

        _enum_ordinal(&self) -> Result<Option<AstLoc<RpValue>>> {
            (_: enum_ordinal, _: equals, value: _value_token()) => value.map(Some),
            () => Ok(None),
        }

        _optional_value_list(&self) -> Result<LinkedList<AstLoc<RpValue>>> {
            (_: optional_value_list, values: _value_list()) => values,
            () => Ok(LinkedList::new()),
        }

        _value_list(&self) -> Result<LinkedList<AstLoc<RpValue>>> {
            (value: _value_token(), _: comma, tail: _value_list()) => {
                let mut tail = tail?;
                tail.push_front(value?);
                Ok(tail)
            },

            (value: _value_token()) => {
                let mut tail = LinkedList::new();
                tail.push_front(value?);
                Ok(tail)
            },
        }

        _value_token(&self) -> Result<AstLoc<RpValue>> {
            (token: value, value: _value()) => {
                let pos = (token.start, token.end);
                value.map(move |v| AstLoc::new(v, pos))
            },
        }

        _value(&self) -> Result<RpValue> {
            (
                token: instance,
                _: custom_type,
                custom: _custom(),
                arguments_token: instance_arguments,
                _: left_paren,
                arguments: _field_init_list(),
                _: right_paren,
            ) => {
                let arguments = arguments?.into_iter().collect();

                let args_pos = (arguments_token.start, arguments_token.end);
                let instance = Instance {
                   ty: custom,
                   arguments: AstLoc::new(arguments, args_pos),
                };

                let pos = (token.start, token.end);
                Ok(RpValue::Instance(AstLoc::new(instance, pos)))
            },

            (
                token: constant,
                _: custom_type,
                custom: _custom(),
            ) => {
                let pos = (token.start, token.end);
                Ok(RpValue::Constant(AstLoc::new(custom, pos)))
            },

            (
                _: array,
                _: bracket_start,
                values: _optional_value_list(),
                _: bracket_end,
            ) => {
                let values = values?.into_iter().collect();
                Ok(RpValue::Array(values))
            },

            (&value: string) => {
                let value = decode_escaped_string(value)?;
                Ok(RpValue::String(value))
            },

            (&value: identifier) => {
                Ok(RpValue::Identifier(value.to_owned()))
            },

            (&value: number) => {
                let value = value.parse::<f64>()?;
                Ok(RpValue::Number(value))
            },

            (&value: boolean) => {
                let value = match value {
                    "true" => true,
                    "false" => false,
                    _ => panic!("should not happen"),
                };

                Ok(RpValue::Boolean(value))
            },
        }

        _used_prefix(&self) -> Option<String> {
            (_: used_prefix, &prefix: identifier, _: scope) => Some(prefix.to_owned()),
            () => None,
        }

        _field_init_list(&self) -> Result<LinkedList<AstLoc<FieldInit>>> {
            (
                token: field_init,
                field_init: _field_init(),
                _: comma,
                tail: _field_init_list()
            ) => {
                let mut tail = tail?;
                tail.push_front(AstLoc::new(field_init?, (token.start, token.end)));
                Ok(tail)
            },

            (
                token: field_init,
                field_init: _field_init(),
                tail: _field_init_list()
            ) => {
                let mut tail = tail?;
                tail.push_front(AstLoc::new(field_init?, (token.start, token.end)));
                Ok(tail)
            },

            () => Ok(LinkedList::new()),
        }

        _field_init(&self) -> Result<FieldInit> {
            (
                name_token: field_name,
                &name: identifier,
                _: colon,
                value: _value_token(),
            ) => {
                Ok(FieldInit {
                    name: AstLoc::new(name.to_owned(), (name_token.start, name_token.end)),
                    value: value?,
                })
            },
        }

        _member_list(&self) -> Result<LinkedList<AstLoc<Member>>> {
            (token: member, value: _member(), tail: _member_list()) => {
                let mut tail = tail?;
                let pos = (token.start, token.end);
                tail.push_front(AstLoc::new(value?, pos));
                Ok(tail)
            },

            () => Ok(LinkedList::new()),
        }

        _member(&self) -> Result<Member> {
            (
                _: field,
                &name: identifier,
                modifier: _modifier(),
                _: colon,
                type_spec: _type_spec(),
                field_as: _field_as(),
                _: semi_colon,
            ) => {
                let field = Field {
                    modifier: modifier,
                    name: name.to_owned(),
                    ty: type_spec?,
                    field_as: field_as?,
                };

                Ok(Member::Field(field))
            },

            (
                _: code_block,
                &context: identifier,
                _: code_start,
                &content: code_body,
                _: code_end,
             ) => {
                let block = strip_code_block(content);
                Ok(Member::Code(context.to_owned(), block))
            },

            (
                token: option_decl,
                &name: identifier,
                values: _value_list(),
                _: semi_colon,
            ) => {
                let pos = (token.start, token.end);
                let values = values?.into_iter().collect();
                let option_decl = OptionDecl { name: name.to_owned(), values: values };
                Ok(Member::Option(AstLoc::new(option_decl, pos)))
            },

            (
                _: match_decl,
                _: match_keyword,
                _: left_curly,
                members: _match_member_list(),
                _: right_curly,
             ) => {
                let members = members?.into_iter().collect();

                let decl = MatchDecl {
                    members: members,
                };

                Ok(Member::Match(decl))
            },
        }

        _field_as(&self) -> Result<Option<AstLoc<RpValue>>> {
            (_: field_as, _: as_keyword, value: _value_token()) => Ok(Some(value?)),
            () => Ok(None),
        }

        _sub_type_list(&self) -> Result<LinkedList<AstLoc<SubType>>> {
            (token: sub_type, value: _sub_type(), tail: _sub_type_list()) => {
                let mut tail = tail?;
                let pos = (token.start, token.end);
                tail.push_front(AstLoc::new(value?, pos));
                Ok(tail)
            },

            () => {
                Ok(LinkedList::new())
            },
        }

        _sub_type(&self) -> Result<SubType> {
            (
                &name: type_identifier,
                _: left_curly,
                members: _member_list(),
                _: right_curly,
             ) => {
                let name = name.to_owned();
                let members = members?.into_iter().collect();
                Ok(SubType { name: name, members: members })
            },
        }

        _match_member_list(&self) -> Result<LinkedList<AstLoc<MatchMember>>> {
            (
                _: match_member_entry,
                member: _match_member(),
                tail: _match_member_list(),
            ) => {
                let mut tail = tail?;
                tail.push_front(member?);
                Ok(tail)
            },

            () => Ok(LinkedList::new()),
        }

        _match_member(&self) -> Result<AstLoc<MatchMember>> {
            (
                token: match_member,
                condition: _match_condition(),
                _: hash_rocket,
                value: _value_token(),
                _: semi_colon,
            ) => {
                let pos = (token.start, token.end);

                let member = MatchMember {
                    condition: condition?,
                    value: value?,
                };

                Ok(AstLoc::new(member, pos))
            },
        }

        _match_condition(&self) -> Result<AstLoc<MatchCondition>> {
            (
                token: match_condition,
                _: match_value,
                value: _value_token(),
            ) => {
                let pos = (token.start, token.end);
                let value = value?;
                let condition = MatchCondition::RpValue(value);
                Ok(AstLoc::new(condition, pos))
            },

            (
                token: match_condition,
                match_token: match_variable,
                &name: identifier,
                _: colon,
                ty: _type_spec(),
            ) => {
                let pos = (token.start, token.end);
                let name = name.to_owned();
                let ty = ty?;

                let variable = MatchVariable {
                    name: name,
                    ty: ty,
                };

                let variable = AstLoc::new(variable, (match_token.start, match_token.end));

                let condition = MatchCondition::Type(variable);

                Ok(AstLoc::new(condition, pos))
            },
        }

        _type_spec(&self) -> Result<RpType> {
            (_: double_type) => {
                Ok(RpType::Double)
            },

            (_: float_type) => {
                Ok(RpType::Float)
            },

            (_: signed_type, _: forward_slash, &size: unsigned) => {
                let size = size.parse::<usize>()?;
                Ok(RpType::Signed(Some(size)))
            },

            (_: unsigned_type, _: forward_slash, &size: unsigned) => {
                let size = size.parse::<usize>()?;
                Ok(RpType::Unsigned(Some(size)))
            },

            (_: signed_type) => {
                Ok(RpType::Signed(None))
            },

            (_: unsigned_type) => {
                Ok(RpType::Unsigned(None))
            },

            (_: boolean_type) => {
                Ok(RpType::Boolean)
            },

            (_: string_type) => {
                Ok(RpType::String)
            },

            (_: bytes_type) => {
                Ok(RpType::Bytes)
            },

            (_: any_type) => {
                Ok(RpType::Any)
            },

            (_: array_type, _: bracket_start, argument: _type_spec(), _: bracket_end) => {
                let argument = argument?;
                Ok(RpType::Array(Box::new(argument)))
            },

            (
                _: map_type,
                _: left_curly,
                key: _type_spec(),
                _: colon,
                value: _type_spec(),
                _: right_curly
             ) => {
                let key = key?;
                let value = value?;
                Ok(RpType::Map(Box::new(key), Box::new(value)))
            },

            (_: custom_type, custom: _custom()) => {
                Ok(RpType::Name(custom))
            },
        }

        _custom(&self) -> RpName {
            (prefix: _used_prefix(), parts: _type_identifier_list()) => {
                let parts = parts.into_iter().collect();

                RpName {
                    prefix: prefix,
                    parts: parts,
                }
            },
        }

        _modifier(&self) -> RpModifier {
            (_: optional) => RpModifier::Optional,
            () => RpModifier::Required,
        }

        _ident_list(&self) -> LinkedList<String> {
            (&value: identifier, _: dot, mut tail: _ident_list()) => {
                tail.push_front(value.to_owned());
                tail
            },

            (&value: identifier, mut tail: _ident_list()) => {
                tail.push_front(value.to_owned());
                tail
            },

            () => LinkedList::new(),
        }

        _type_identifier_list(&self) -> LinkedList<String> {
            (&value: type_identifier, _: dot, mut tail: _type_identifier_list()) => {
                tail.push_front(value.to_owned());
                tail
            },

            (&value: type_identifier) => {
                let mut tail = LinkedList::new();
                tail.push_front(value.to_owned());
                tail
            },

        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check that a parsed value equals expected.
    macro_rules! assert_value_eq {
        ($expected:expr, $input:expr) => {{
            let mut parser = parse($input);
            assert!(parser.value());

            let v = parser._value_token().unwrap().inner;
            assert_eq!($expected, v);
        }}
    }

    macro_rules! assert_type_spec_eq {
        ($expected:expr, $input:expr) => {{
            let mut parser = parse($input);
            assert!(parser.type_spec());
            assert!(parser.end());

            let v = parser._type_spec().unwrap();
            assert_eq!($expected, v);
        }}
    }

    const FILE1: &[u8] = include_bytes!("tests/file1.reproto");
    const INTERFACE1: &[u8] = include_bytes!("tests/interface1.reproto");

    fn parse(input: &'static str) -> Rdp<StringInput> {
        Rdp::new(StringInput::new(input))
    }

    #[test]
    fn test_file1() {
        let input = ::std::str::from_utf8(FILE1).unwrap();
        let mut parser = parse(input);

        assert!(parser.file());
        assert!(parser.end());

        let file = parser._file().unwrap();

        let package = RpPackage::new(vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]);

        assert_eq!(package, *file.package);
        assert_eq!(4, file.decls.len());
    }

    #[test]
    fn test_array() {
        let mut parser = parse("[string]");

        assert!(parser.type_spec());
        assert!(parser.end());

        let ty = parser._type_spec().unwrap();

        if let RpType::Array(inner) = ty {
            if let RpType::String = *inner {
                return;
            }
        }

        panic!("Expected Type::Array(Type::String)");
    }

    #[test]
    fn test_map() {
        let mut parser = parse("{string: unsigned/123}");

        assert!(parser.type_spec());
        assert!(parser.end());

        let ty = parser._type_spec().unwrap();

        // TODO: use #![feature(box_patterns)]:
        // if let Type::Map(box Type::String, box Type::Unsigned(size)) = ty {
        // }
        if let RpType::Map(key, value) = ty {
            if let RpType::String = *key {
                if let RpType::Unsigned(size) = *value {
                    assert_eq!(Some(123usize), size);
                    return;
                }
            }
        }

        panic!("Expected Type::Array(Type::String)");
    }

    #[test]
    fn test_block_comment() {
        let mut parser = parse("/* hello \n world */");

        assert!(parser.comment());
    }

    #[test]
    fn test_line_comment() {
        let mut parser = parse("// hello world\n");

        assert!(parser.comment());
    }

    #[test]
    fn test_code_block() {
        let mut parser = parse("a { b { c } d } e");

        assert!(parser.code_body());
        assert!(parser.end());
    }

    #[test]
    fn test_code() {
        let mut parser = parse("java{{\na { b { c } d } e\n}}");

        assert!(parser.code_block());
        assert!(parser.end());
    }

    #[test]
    fn test_find_indent() {
        assert_eq!(Some(4), find_indent("   \thello"));
        assert_eq!(Some(0), find_indent("nope"));
        assert_eq!(None, find_indent(""));
        assert_eq!(None, find_indent("    "));
    }

    #[test]
    fn test_strip_code_block() {
        let result = strip_code_block("\n   hello\n  world\n\n\n again\n\n\n");
        assert_eq!(vec!["  hello", " world", "", "", "again"], result);
    }

    #[test]
    fn test_interface() {
        let input = ::std::str::from_utf8(INTERFACE1).unwrap();
        let mut parser = parse(input);

        assert!(parser.file());
        assert!(parser.end());

        let file = parser._file().unwrap();

        assert_eq!(1, file.decls.len());
    }

    #[test]
    fn test_instance() {
        let c = RpName {
            prefix: None,
            parts: vec!["Foo".to_owned(), "Bar".to_owned()],
        };

        let field = FieldInit {
            name: AstLoc::new("hello".to_owned(), (8, 13)),
            value: AstLoc::new(RpValue::Number(12f64), (15, 17)),
        };

        let field = AstLoc::new(field, (8, 17));

        let instance = Instance {
            ty: c,
            arguments: AstLoc::new(vec![field], (7, 18)),
        };

        assert_value_eq!(RpValue::Instance(AstLoc::new(instance, (0, 18))),
                         "Foo.Bar(hello: 12)");
    }

    #[test]
    fn test_values() {
        assert_value_eq!(RpValue::String("foo\nbar".to_owned()), "\"foo\\nbar\"");
        assert_value_eq!(RpValue::Number(1f64), "1");
        assert_value_eq!(RpValue::Number(1.25f64), "1.25");
    }

    #[test]
    fn test_type_spec() {
        let c = RpName {
            prefix: None,
            parts: vec!["Hello".to_owned(), "World".to_owned()],
        };

        assert_type_spec_eq!(RpType::String, "string");
        assert_type_spec_eq!(RpType::Name(c), "Hello.World");
    }

    #[test]
    fn test_option_decl() {
        let mut parser = parse("foo_bar_baz true, foo, \"bar\", 12;");

        assert!(parser.option_decl());
        assert!(parser.end());

        if let Member::Option(option) = parser._member().unwrap() {
            assert_eq!("foo_bar_baz", option.name);
            assert_eq!(4, option.values.len());

            assert_eq!(RpValue::Boolean(true), option.values[0].inner);
            assert_eq!(RpValue::Identifier("foo".to_owned()),
                       option.values[1].inner);
            assert_eq!(RpValue::String("bar".to_owned()), option.values[2].inner);
            assert_eq!(RpValue::Number(12f64), option.values[3].inner);
            return;
        }

        panic!("option did not match");
    }
}
