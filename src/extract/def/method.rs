use extract::def::type_param;
use extract::{block, Definition, Overlay};
use parse::tree::Method;

pub fn apply<'def, 'def_ref, 'overlay_ref>(
    method: &'def_ref Method<'def>,
    overlay: &'overlay_ref mut Overlay<'def>,
) {
    if let Some(def) = method.def_opt.borrow().as_ref() {
        overlay.defs.push(Definition::Method(*def));
    }

    for t in &method.type_params {
        type_param::apply(t, overlay);
    }

    if let Some(b) = &method.block_opt {
        block::apply(b, overlay);
    }
}
