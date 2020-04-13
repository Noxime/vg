use vg::*;

#[magic::vg]
async fn main<A: Api>(mut api: A) {
    println!("Hello, world!");
}