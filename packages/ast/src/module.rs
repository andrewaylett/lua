use crate::func::SourceFunction;
use crate::name::Name;
use crate::{expect_rule, AstError, AstResult};
use gor_parse::Rule;
use pest::iterators::{Pair, Pairs};
use std::collections::HashMap;
use std::fmt::Debug;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SourceModule<'i> {
    pub package: Name,
    imports: Vec<Name>,
    functions: HashMap<Name, Box<SourceFunction<'i>>>,
}

impl<'i> TryFrom<Pairs<'i, Rule>> for SourceModule<'i> {
    type Error = AstError;

    fn try_from(mut pairs: Pairs<'i, Rule>) -> super::AstResult<Self> {
        let pair = pairs.next().ok_or(AstError::InvalidState(
            "Expected to get a module, but found nothing to parse",
        ))?;
        let item = SourceModule::try_from(pair);
        if pairs.next().is_some() {
            Err(AstError::InvalidState(
                "Expected to consume all of the parse",
            ))
        } else {
            item
        }
    }
}

impl<'i> TryFrom<Pair<'i, Rule>> for SourceModule<'i> {
    type Error = AstError;

    fn try_from(module: Pair<'i, Rule>) -> AstResult<Self> {
        expect_rule(&module, Rule::module)?;
        primary(module)
    }
}

impl<'i> SourceModule<'i> {
    pub fn function(&self, name: &str) -> Option<&SourceFunction<'i>> {
        let name = name.into();
        self.functions.get(&name).map(|b| b.as_ref())
    }
}

fn primary<'i>(module: Pair<'i, Rule>) -> AstResult<SourceModule<'i>> {
    expect_rule(&module, Rule::module)?;

    let inner: Pairs<'i, Rule> = module.into_inner();
    let mut package = None;
    let mut imports = vec![];
    let mut functions: HashMap<Name, Box<SourceFunction<'i>>> = HashMap::new();
    for pair in inner {
        match pair.as_rule() {
            Rule::package => {
                let name = pair
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::name)
                    .ok_or(AstError::InvalidState(
                        "Found a package declaration without a name",
                    ))?;
                package = Some(Name::from(name.as_str()));
            }
            Rule::import => {
                let string = pair
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::string)
                    .ok_or(AstError::InvalidState("Found an import without a package"))?;
                let name = string
                    .into_inner()
                    .find(|p| p.as_rule() == Rule::string_inner)
                    .ok_or(AstError::InvalidState(
                        "Found an import string without an inner",
                    ))?;
                imports.push(Name::from(name.as_str()));
            }
            Rule::func => {
                let func = SourceFunction::try_from(pair)?;
                functions.insert(func.name, Box::new(func));
            }
            Rule::EOI => {}
            r => return Err(AstError::InvalidRule("module contents", r)),
        }
    }
    match package {
        None => Err(AstError::InvalidState("Module must have package set")),
        Some(package) => Ok(SourceModule {
            package,
            imports,
            functions,
        }),
    }
}
