# Setup
Please create a `data` folder at the root of the project, containing `program.json` and `trace.csv`.

# Running the code
By typing:
```
cargo run
```
two files are generated:
- `flat_call_map.json`: contains entries are the functions of the program. Each function has a number of proper instructions (`proper_inst`),
and a number of cumulative instructions, which is the sum of the proper instructions of all its callees.
- `call_tree.json`: contains the same data as in the previous file, but within a tree structure. A node is a function, and its children are its callees.

The `call_tree.json` could be used to visualize the program as a call graph (as with the call graph tool of Valgrind).