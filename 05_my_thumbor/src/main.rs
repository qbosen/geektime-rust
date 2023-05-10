mod pb;

struct ImageSpec {
    specs: Vec<Spec>,
}
enum Spec {
    Resize(Resize),
    Crop(Crop),
}

struct Resize {
    width: u32,
    height: u32,
}
struct Crop {}

fn main() {
    println!("Hello, world!");
}
