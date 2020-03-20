extern crate varlink_generator;

fn main() {
    varlink_generator::cargo_build_tosource(
        "src/info.grisge.zatel.varlink",
        true,
    );
}
