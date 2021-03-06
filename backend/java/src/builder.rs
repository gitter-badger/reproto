//! Module that adds fasterxml annotations to generated classes.

use backend::errors::*;
use genco::{Java, Quoted, Tokens};
use genco::java::{Argument, Class, Field, Method, Modifier, imported, local};
use listeners::{ClassAdded, Listeners};
use std::rc::Rc;

pub struct Module {
    optional: Java<'static>,
    runtime_exception: Java<'static>,
}

impl Module {
    pub fn new() -> Module {
        Module {
            optional: imported("java.util", "Optional"),
            runtime_exception: imported("java.lang", "RuntimeException"),
        }
    }
}

impl Module {
    fn builder_field<'el>(&self, field: &Field<'el>) -> Field<'el> {
        use self::Modifier::*;

        let ty = match field.ty() {
            optional @ Java::Optional(_) => optional,
            other => self.optional.with_arguments(vec![other]),
        };

        let mut field = Field::new(ty, field.var());
        field.modifiers = vec![Private];
        field.initializer(toks![self.optional.clone(), ".empty()"]);
        field
    }

    fn setter_method<'el>(&self, field: &Field<'el>) -> Method<'el> {
        let argument = Argument::new(field.ty().as_value(), field.var());

        let mut setter = Method::new(field.var());
        setter.returns = local("Builder");
        setter.arguments.push(argument.clone());

        setter.body.push(toks![
            "this.",
            field.var(),
            " = ",
            self.optional.clone(), ".of(", argument.var(), ")",
            ";",
        ]);
        setter.body.push("return this;");

        setter
    }
}

impl Listeners for Module {
    fn class_added(&self, e: &mut ClassAdded) -> Result<()> {
        use self::Modifier::*;

        let mut builder = Class::new("Builder");
        builder.modifiers = vec![Public, Static];

        let mut build_variable_assign = Tokens::new();
        let mut build_constructor_arguments = Tokens::new();

        for field in &e.spec.fields {
            builder.fields.push(self.builder_field(field));
            builder.methods.push(self.setter_method(field));

            let value = if !field.ty().is_optional() {
                let message = Rc::new(format!("{}: is required", field.var().as_ref())).quoted();
                let throw_toks = toks!["new ", self.runtime_exception.clone(), "(", message, ")"];

                toks!["this.", field.var(), ".orElseThrow(() -> ", throw_toks, ")"]
            } else {
                toks!["this.", field.var()]
            };

            let assign: Tokens<Java> =
                toks!["final ", field.ty(), " ", field.var(), " = ", value, ";"];

            build_variable_assign.push(assign);
            build_constructor_arguments.append(field.var());
        }

        builder.methods.push({
            let mut build = Method::new("build");
            build.returns = local(e.spec.name());

            build.body.push(build_variable_assign);

            build.body.push(toks![
                "return new ",
                e.spec.name(),
                "(",
                build_constructor_arguments.join(", "),
                ");",
            ]);

            build.body = build.body.join_line_spacing();
            build
        });

        e.spec.body.push(builder);
        Ok(())
    }
}
