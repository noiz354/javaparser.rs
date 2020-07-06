#![feature(box_patterns)]

extern crate javaparser;
extern crate log4rs;
extern crate log;



use log::{error, info, warn};
// use log4rs;

use javaparser::{parse, tokenize};
use std::fs;
use std::time::Instant;

fn main() {

        log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

        let path = "/Users/normansyahputa/lalala/hansel plugin groovy/hansel_plugin_groovy/src/main/java/p000io/hansel/ProjectUtils.java";
        let content = fs::read_to_string(path).unwrap();
    
        let start = Instant::now();
        let tokens = tokenize::apply(&content, std::ptr::null()).ok().unwrap();
        let result = parse::apply(&content, &path);
        let elapsed = start.elapsed().as_nanos();

        info!("booting up");
        error!("testing");
        warn!("testing");

        let result = result.unwrap();

        algo(&result.unit);


        // println!("{:?}",result);
    
        // println!("Parsing took {:?} (succeed: {})", elapsed, result.is_ok());
}

use parse::tree::CompilationUnit;
use parse::tree::CompilationUnitItem;

// use std::any::type_name;


// fn type_of<T>(_: T) -> &'static str {
//         type_name::<T>()
// }

// use std::any::{Any, TypeId};

// fn is_string<T: ?Sized + Any>(_s: &T) -> bool {
//     TypeId::of::<String>() == TypeId::of::<T>()
// }

// fn is_class<'def, T: ?Sized + Any>(_s: &'def T) -> bool {
//         TypeId::of::<CompilationUnitItem::Class>() == TypeId::of::<T>()
// }

fn algo(cu: &CompilationUnit){
        let mut iter = cu.items.iter();
        if let Some(result) = iter.find(|&x| match x {
                CompilationUnitItem::Class(ref x) => true,
                _ => return false,
        }){
                match result {
                        CompilationUnitItem::Class(ref x)  => {
                                recu_class(x);
                        },
                        _ => {},
                }
        }
}

use parse::tree::Class;
use parse::tree::ClassBodyItem;
use parse::tree::Statement;
use parse::tree::Expr;

// find constructor caller
// recursively find and print class name 
fn recu_class(cu: &Class){
        println!("class inside {:?}", cu.name.fragment);

        for body_item in &cu.body.items {
                match body_item {
                        ClassBodyItem::Method(ref y) => {
                                if y.name.fragment == "setDependencies" {
                                        println!("method {:?} in class {:?}", y.name.fragment, cu.name.fragment);
                                        if let Some(x) = &y.block_opt {
                                                for stmt in &x.stmts{
                                                        match stmt {
                                                                Statement::Expr(ref x1) => {
                                                                        match x1 {
                                                                                Expr::ConstructorReference(ref x2) => {
                                                                                        println!("constructor reference {:?}", x2);
                                                                                },
                                                                                Expr::Assignment(ref x3) => {
                                                                                        println!("constructor reference {:?}", x3);
                                                                                },
                                                                                _ => {
                                                                                        println!("other");
                                                                                },
                                                                        }
                                                                },
                                                                Statement::VariableDeclarators(ref x1) => {
                                                                        println!("\n\n\n\n\n{:?}", x1.declarators);
                                                                },
                                                                Statement::IfElse(ref x1) => {
                                                                        let get_this = match x1.cond { 
                                                                                Expr::MethodCall(ref x2) => {
                                                                                        match &x2.prefix_opt  {
                                                                                                Some(box x) => {
                                                                                                        // let x = x.to_owned();
                                                                                                        match x {
                                                                                                                Expr::Name(ref x3) => {
                                                                                                                        format!("{}.{}", x3.name.fragment, x2.name.fragment)
                                                                                                                },
                                                                                                                _ => "".to_string()
                                                                                                        }
                                                                                                },
                                                                                                _ => {
                                                                                                        "".to_string()
                                                                                                },
                                                                                        }
                                                                                },
                                                                                _ => "".to_string()
                                                                        };
                                                                        println!("\n\try_this\n\n\n\n\n{:?}", get_this);
                                                                        println!("\n\ncond\n\n\n\n\n{:?}", x1.cond);
                                                                        println!("\n\nblock\n\n\n\n\n{:?}", x1.block);
                                                                }
                                                                _ => {},
                                                        }
                                                }
                                        };
                                }
                                // true
                        },
                        // _ => false,
                        _ => {},
                }
        }

        // let mut iter = cu.body.items.iter();
        // if let Some(result) = iter.find(|&x| match x {
        //         ClassBodyItem::Method(ref y) => {
        //                 println!("method {:?} in class {:?}", y.name.fragment, cu.name.fragment);
        //                 if y.name.fragment == "setDependencies" {
        //                         if let Some(x) = &y.block_opt {
        //                                 for stmt in &x.stmts{
        //                                         match stmt {
        //                                                 Statement::Expr(ref x1) => {
        //                                                         match x1 {
        //                                                                 Expr::ConstructorReference(ref x2) => {
        //                                                                         println!("constructor reference {:?}", x2);
        //                                                                 }
        //                                                                 _ => {},
        //                                                         }
        //                                                 },
        //                                                 _ => {},
        //                                         }
        //                                 }
        //                         };
        //                 }
        //                 true
        //         },
        //         _ => false,
        // }){

        // };
        // if let Some(result) = iter.find(|&x| match x {
        //         ClassBodyItem::Class(ref x) => true,
        //         _ => return false,
        // }){
        //         match result {
        //                 ClassBodyItem::Class(ref x)  => {
        //                         recu_class(x);
        //                         // find constructor
        //                         for class_body in &x.body.items {
        //                                 match class_body {
        //                                         ClassBodyItem::StaticInitializer(ref y) => {
        //                                                 // println!("x {:?} y {:?}", x.name.fragment, y.name.fragment);
        //                                                 // if y.name.fragment == x.name.fragment {
        //                                                 //         println!("ketemu konstruktornya  {:?}", y.name.fragment);
        //                                                 // } 
        //                                         },   
        //                                         ClassBodyItem::Constructor(ref y) => {
        //                                                 println!("x {:?} y {:?}", x.name.fragment, y.name.fragment);
        //                                                 if y.name.fragment == x.name.fragment {
        //                                                         println!("ketemu konstruktornya  {:?}", y.name.fragment);
        //                                                 } 
        //                                         },
        //                                         ClassBodyItem::Method(ref y) => {
        //                                                 println!("x {:?} y {:?}", x.name.fragment, y.name.fragment);
        //                                                 if y.name.fragment == x.name.fragment {
        //                                                         println!("ketemu konstruktornya  {:?}", y.name.fragment);
        //                                                 } 
        //                                         },
        //                                         _ => {},
        //                                 }
        //                         }
        //                 },
        //                 _ => {},
        //         } 
        // }
}
