// This file is used for normal running, but due to platforms that dont support
// rust we have to compile KEA as a static lib and bootstrap it with a language
// that does work (TLDR; iOS is dumb dumb)

extern crate kea;

use kea::kea_run;

fn main() {
    // println!("YOO WE RUNNING FROM MAIN U KNOW");
    kea_run();
}
