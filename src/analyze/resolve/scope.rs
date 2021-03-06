use analyze::definition::{Class, Decl, Method, MethodDef, Package, Param, Root, TypeParam};
use parse::tree::{
    ClassType, EnclosingType, ImportPrefix, InvocationContext, PackagePrefix, ParameterizedType,
    ResolvedName, VariableDeclarator, NATIVE_ARRAY_CLASS_NAME,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use tokenize::span::Span;
use {analyze, parse};

#[derive(Debug, PartialEq, Clone)]
pub struct Scope<'def, 'r>
where
    'def: 'r,
{
    pub root: &'r Root<'def>,
    pub levels: Vec<Level<'def>>,
    pub specific_imports: Vec<SpecificImport<'def>>,
    pub wildcard_imports: Vec<WildcardImport<'def>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SpecificImport<'def> {
    pub class: *const Class<'def>,
    pub is_static: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WildcardImport<'def> {
    pub enclosing: EnclosingTypeDef<'def>,
    pub is_static: bool,
}
unsafe impl<'def> Send for WildcardImport<'def> {}

#[derive(Debug, PartialEq, Clone)]
pub struct Level<'def> {
    pub enclosing_opt: Option<EnclosingTypeDef<'def>>,
    pub names: HashMap<String, Vec<Name<'def>>>,
}
unsafe impl<'def> Send for Level<'def> {}

#[derive(Debug, PartialEq, Clone)]
pub enum Name<'def> {
    TypeParam(*const TypeParam<'def>),
    Variable(*const VariableDeclarator<'def>),
    Param(*const Param<'def>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum EnclosingTypeDef<'def> {
    Package(*const Package<'def>),
    Class(*const Class<'def>),
}

impl<'def> EnclosingTypeDef<'def> {
    pub fn find(&self, name: &str) -> Option<EnclosingTypeDef<'def>> {
        match self {
            EnclosingTypeDef::Package(package) => unsafe { (**package).find(name) },
            EnclosingTypeDef::Class(class) => {
                unsafe { (**class).find(name) }.map(|r| EnclosingTypeDef::Class(r))
            }
        }
    }
    pub fn find_package<'a, 'b>(&'a self, name: &str) -> Option<&'b Package<'def>> {
        match self.find(name) {
            Some(EnclosingTypeDef::Package(package)) => Some(unsafe { &*package }),
            Some(EnclosingTypeDef::Class(_)) => panic!(),
            None => None,
        }
    }
    pub fn find_class<'a, 'b>(&'a self, name: &str) -> Option<&'b Class<'def>> {
        match self.find(name) {
            Some(EnclosingTypeDef::Package(_)) => panic!(),
            Some(EnclosingTypeDef::Class(class)) => Some(unsafe { &*class }),
            None => None,
        }
    }

    pub fn to_type(&self) -> EnclosingType<'def> {
        match self {
            EnclosingTypeDef::Package(package) => {
                let package = unsafe { &(**package) };
                EnclosingType::Package(PackagePrefix {
                    prefix_opt: None,
                    name: package.name.to_owned(),
                    span_opt: None,
                    def: package as *const Package,
                })
            }
            EnclosingTypeDef::Class(class) => {
                let class = unsafe { &(**class) };
                EnclosingType::Class(class.to_type())
            }
        }
    }
}

impl<'def, 'r> Scope<'def, 'r> {
    fn resolve_import_prefix<'s>(&self, prefix: &'s ImportPrefix<'def>) -> EnclosingTypeDef<'def> {
        match &prefix.prefix_opt {
            None => self.root.find(prefix.name.fragment).unwrap(),
            Some(prefix_prefix) => self.resolve_import_prefix(prefix_prefix),
        }
    }

    pub fn add_import<'s>(&mut self, import: &'s parse::tree::Import<'def>) {
        let enclosing_prefix_opt = match &import.prefix_opt {
            Some(prefix) => Some(self.resolve_import_prefix(prefix)),
            None => None,
        };

        let enclosing = match enclosing_prefix_opt {
            Some(prefix) => self
                .resolve_type_def_at(&prefix, import.name.fragment)
                .unwrap(),
            None => self.root.find(import.name.fragment).unwrap(),
        };

        if import.is_wildcard {
            self.wildcard_imports.push(WildcardImport {
                enclosing,
                is_static: import.is_static,
            });
        } else {
            let class = if let EnclosingTypeDef::Class(class) = enclosing {
                class
            } else {
                return;
            };

            self.specific_imports.push(SpecificImport {
                class,
                is_static: import.is_static,
            });
        };
    }

    fn add_name(&mut self, name: &str, value: Name<'def>) {
        let level = self.levels.last_mut().unwrap();

        if let None = level.names.get_mut(name) {
            level.names.insert(String::from(name), vec![]);
        }

        let list = level.names.get_mut(name).unwrap();

        list.insert(0, value)
    }

    pub fn add_type_param<'s>(&mut self, type_param: &'s TypeParam<'def>) {
        self.add_name(
            &type_param.name,
            Name::TypeParam(type_param as *const TypeParam<'def>),
        );
    }

    pub fn add_param<'s>(&mut self, param: &'s Param<'def>) {
        self.add_name(
            param.name.fragment,
            Name::Param(param as *const Param<'def>),
        );
    }

    pub fn add_variable<'s>(&mut self, variable: &'s VariableDeclarator<'def>) {
        self.add_name(
            variable.name.fragment,
            Name::Variable(variable as *const VariableDeclarator<'def>),
        );
    }

    pub fn enter_package(&mut self, package: &'r Package<'def>) {
        self.levels.push(Level {
            enclosing_opt: Some(EnclosingTypeDef::Package(package)),
            names: HashMap::new(),
        });
    }

    pub fn leave(&mut self) {
        self.levels.pop();
    }

    pub fn enter_class(&mut self, class: &'r Class<'def>) {
        self.levels.push(Level {
            enclosing_opt: Some(EnclosingTypeDef::Class(class)),
            names: HashMap::new(),
        });
    }

    pub fn enter(&mut self) {
        self.levels.push(Level {
            enclosing_opt: None,
            names: HashMap::new(),
        });
    }

    pub fn resolve_methods(&self, name: &str, context: &InvocationContext) -> Vec<Method<'def>> {
        let mut methods = vec![];
        for i in 0..self.levels.len() {
            let current = self.levels.get(self.levels.len() - 1 - i).unwrap();

            if let Some(enclosing) = &current.enclosing_opt {
                match enclosing {
                    EnclosingTypeDef::Package(_) => break,
                    EnclosingTypeDef::Class(c) => methods
                        .append(&mut unsafe { &**c }.to_type().find_methods(name, context, 0)),
                };
            }
        }

        methods
    }

    pub fn resolve_package(&self, name: &str) -> Option<*const Package<'def>> {
        self.root
            .find_package(name)
            .map(|p| p as *const Package<'def>)
    }

    // Resolve a name (e.g. type param, variable, arg, class, package) in a method's body
    pub fn resolve_name(&self, name: &str) -> Option<ResolvedName<'def>> {
        for i in 0..self.levels.len() {
            let current = self.levels.get(self.levels.len() - 1 - i).unwrap();

            if let Some(locals) = current.names.get(name) {
                for local in locals {
                    match local {
                        Name::TypeParam(type_param) => {
                            return Some(ResolvedName::TypeParam(*type_param));
                        }
                        Name::Variable(v) => {
                            return Some(ResolvedName::Variable(*v));
                        }
                        Name::Param(p) => {
                            return Some(ResolvedName::Param(*p));
                        }
                    }
                }
            }

            // TODO: We should support field here.

            if let Some(enclosing) = &current.enclosing_opt {
                if let Some(result) = self.resolve_type_at(enclosing, name) {
                    return match result {
                        EnclosingType::Package(p) => Some(ResolvedName::Package(p.def)),
                        EnclosingType::Class(c) => Some(ResolvedName::Class(c.def_opt.unwrap())),
                        EnclosingType::Parameterized(_) => panic!(),
                    };
                }

                // We search until the closest package. Java only allows referring to a package using its full path.
                // There's no such thing as a relative path for package.
                if let EnclosingTypeDef::Package(_) = enclosing {
                    break;
                }
            }
        }
        self.root.find(name).map(|e| match e {
            EnclosingTypeDef::Package(p) => ResolvedName::Package(p),
            EnclosingTypeDef::Class(c) => ResolvedName::Class(c),
        })
    }

    // Resolve a type (e.g. type param, class, package) for or method's return type, field's type, super class type, and etc.
    pub fn resolve_type(&self, name: &str) -> Option<EnclosingType<'def>> {
        for i in 0..self.levels.len() {
            let current = self.levels.get(self.levels.len() - 1 - i).unwrap();

            if let Some(locals) = current.names.get(name) {
                for local in locals {
                    match local {
                        Name::TypeParam(type_param) => {
                            let type_param = unsafe { &(**type_param) };
                            return Some(EnclosingType::Parameterized(ParameterizedType {
                                name: name.to_owned(),
                                span_opt: None,
                                def: type_param,
                            }));
                        }
                        _ => (),
                    }
                }
            }

            if let Some(enclosing) = &current.enclosing_opt {
                if let Some(result) = self.resolve_type_at(enclosing, name) {
                    return Some(result);
                }

                // We search until the closest package. Java only allows referring to a package using its full path.
                // There's no such thing as a relative path for package.
                if let EnclosingTypeDef::Package(_) = enclosing {
                    break;
                }
            }
        }

        if let Some(result) = self.resolve_type_with_specific_import(name) {
            return Some(result);
        }

        if let Some(found) = self
            .root
            .find_package("java")
            .and_then(|p| p.find_package("lang"))
            .and_then(|p| p.find(name))
            .map(|e| e.to_type())
        {
            return Some(found);
        }

        self.root.find(name).map(|e| e.to_type())
    }

    pub fn resolve_type_with_specific_import(&self, name: &str) -> Option<EnclosingType<'def>> {
        for import in &self.specific_imports {
            let class = unsafe { &(*import.class) };
            if &class.name == name {
                return Some(EnclosingType::Class(class.to_type()));
            }
        }

        None
    }

    pub fn resolve_type_at(
        &self,
        current: &EnclosingTypeDef<'def>,
        name: &str,
    ) -> Option<EnclosingType<'def>> {
        match current {
            EnclosingTypeDef::Package(package) => {
                let package = unsafe { &(**package) };

                if let Some(enclosing) = package.find(name) {
                    return Some(enclosing.to_type());
                }
            }
            EnclosingTypeDef::Class(class) => {
                let class = unsafe { &(**class) }.to_type();

                // TODO: the result needs to inherit type args from `class`.
                // Because `class` is the enclosing type def.
                if let Some(found) = class.find_inner_class(name) {
                    return Some(EnclosingType::Class(found));
                }
            }
        };

        None
    }

    pub fn resolve_type_def_at(
        &self,
        current: &EnclosingTypeDef<'def>,
        name: &str,
    ) -> Option<EnclosingTypeDef<'def>> {
        match current {
            EnclosingTypeDef::Package(package) => {
                if let Some(found) = unsafe { &(**package) }.find(name) {
                    return Some(found);
                }
            }
            EnclosingTypeDef::Class(class) => {
                let class = unsafe { &(**class) };
                for decl in &class.decls {
                    if let Decl::Class(subclass) = decl {
                        if &subclass.name == name {
                            return Some(EnclosingTypeDef::Class(subclass.deref()));
                        }
                    }
                }
            }
        };

        None
    }
}
