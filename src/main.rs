use cairo_trace_prof::{
    call_map::CallMap,
    utils::{load_program, load_trace},
};
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    println!("Loading trace...");
    let trace = load_trace("./data/trace.csv").unwrap();
    println!("Loading program...");
    let program = load_program("./data/program.json").unwrap();
    println!("Constructing call map...");
    let callmap = CallMap::new(&program, &trace);
    println!("Checking that all instructions have been counted...");
    assert_eq!(
        trace.len() as u32,
        callmap
            .cmap
            .values()
            .map(|f| f.borrow().proper_inst)
            .sum::<u32>()
    );
    println!("Saving map...");
    let data = serde_json::to_string_pretty(&callmap).unwrap();
    let mut file = File::create("flat_call_map.json")?;
    file.write_all(data.as_bytes())?;
    println!("Computing call tree...");
    let tree = callmap.call_tree();
    println!("Saving tree...");
    let data = serde_json::to_string_pretty(&tree).unwrap();
    let mut file = File::create("call_tree.json")?;
    file.write_all(data.as_bytes())?;
    Ok(())
}
