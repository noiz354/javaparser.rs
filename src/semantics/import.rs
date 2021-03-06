use analyze::resolve::scope::{EnclosingTypeDef, Scope};
use parse::tree::{ImportDef, ImportPrefix, ImportPrefixDef};
use semantics::Context;
use {analyze, parse};

pub fn apply<'def>(import: &mut parse::tree::Import<'def>, context: &mut Context<'def, '_, '_>) {
    match &mut import.prefix_opt {
        Some(prefix) => apply_prefix(prefix, context),
        None => (),
    };

    import.def_opt.replace(get_def(
        &mut import.prefix_opt,
        import.name.fragment,
        &mut context.scope,
    ));
}

pub fn apply_prefix<'def>(
    import: &mut parse::tree::ImportPrefix<'def>,
    context: &mut Context<'def, '_, '_>,
) {
    match &mut import.prefix_opt {
        Some(prefix) => apply_prefix(prefix, context),
        None => (),
    };

    import.def_opt.replace(get_prefix_def(
        &import.prefix_opt,
        import.name.fragment,
        &mut context.scope,
    ));
}

fn get_def<'def>(
    prefix_opt: &mut Option<Box<ImportPrefix<'def>>>,
    name: &'def str,
    scope: &mut Scope<'def, '_>,
) -> Option<ImportDef<'def>> {
    let result_opt = get_enclosing_type_def(prefix_opt, name, scope);

    match result_opt {
        None => None,
        Some(EnclosingTypeDef::Package(package)) => Some(ImportDef::Package(package)),
        Some(EnclosingTypeDef::Class(class)) => Some(ImportDef::Class(class)),
    }
}

fn get_prefix_def<'def>(
    prefix_opt: &Option<Box<ImportPrefix<'def>>>,
    name: &'def str,
    scope: &mut Scope<'def, '_>,
) -> Option<ImportPrefixDef<'def>> {
    let result_opt = get_enclosing_type_def(prefix_opt, name, scope);

    match result_opt {
        None => None,
        Some(EnclosingTypeDef::Package(package)) => Some(ImportPrefixDef::Package(package)),
        Some(EnclosingTypeDef::Class(class)) => Some(ImportPrefixDef::Class(class)),
    }
}

fn get_enclosing_type_def<'def>(
    prefix_opt: &Option<Box<ImportPrefix<'def>>>,
    name: &'def str,
    scope: &mut Scope<'def, '_>,
) -> Option<EnclosingTypeDef<'def>> {
    match prefix_opt {
        Some(prefix) => match prefix.def_opt.borrow().as_ref() {
            Some(ImportPrefixDef::Package(package)) => {
                let package = unsafe { &(**package) };
                package.find(name)
            }
            Some(ImportPrefixDef::Class(class)) => {
                let class = unsafe { &(**class) };
                class.find(name).map(|c| EnclosingTypeDef::Class(c))
            }
            None => None,
        },
        None => scope.root.find(name),
    }
}

#[cfg(test)]
mod tests {
    use analyze::test_common::{find_class, find_package};
    use parse::tree::{Import, ImportDef, ImportPrefix, ImportPrefixDef};
    use std::cell::RefCell;
    use std::ops::Deref;
    use test_common::{span, span2};
    use {analyze, semantics};

    #[test]
    fn test() {
        let (files, root) = apply_semantics![
            r#"
package dev;

import dev2.Super;
import static dev2.*;

class Test {}
        "#,
            r#"
package dev2;

class Super {}
        "#
        ];

        assert_eq!(
            files.first().unwrap().unit.imports,
            vec![
                Import {
                    prefix_opt: Some(Box::new(ImportPrefix {
                        prefix_opt: None,
                        name: span2(3, 8, "dev2", files.get(0).unwrap().deref()),
                        def_opt: RefCell::new(Some(ImportPrefixDef::Package(
                            root.find_package("dev2").unwrap()
                        )))
                    })),
                    is_static: false,
                    is_wildcard: false,
                    name: span2(3, 13, "Super", files.get(0).unwrap().deref()),
                    def_opt: RefCell::new(Some(ImportDef::Class(find_class(&root, "dev2.Super"))))
                },
                Import {
                    prefix_opt: None,
                    is_static: true,
                    is_wildcard: true,
                    name: span2(4, 15, "dev2", files.get(0).unwrap().deref()),
                    def_opt: RefCell::new(Some(ImportDef::Package(
                        root.find_package("dev2").unwrap()
                    )))
                },
            ]
        );
    }
}
