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

// recursively find and print class name 
fn recu_class(cu: &Class){
        println!("{:?}", cu.name.fragment);
        let mut iter = cu.body.items.iter();
        if let Some(result) = iter.find(|&x| match x {
                ClassBodyItem::Class(ref x) => true,
                _ => return false,
        }){
                match result {
                        ClassBodyItem::Class(ref x)  => {
                                recu_class(x);
                        },
                        _ => {},
                } 
        }
}
