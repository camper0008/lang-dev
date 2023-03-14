# couch-lang

first independent lang dev project

had to be named something

syntax:

```
// comment
/* multiline
comment */
fn function_name() {
    let a = 5;
    let mut b = 3;
    b += a;
    return b;
}

let c = function_name();
// c += 1; // ERR: mutating a non-mutable variable
let mut c = 8;
// c += 0.5; // ERR: combining an integer with a non-float
```
