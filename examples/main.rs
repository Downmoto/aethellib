use aethellib::loader::load_files;

fn main() {
    let c = load_files(&[
        "data/weapon_merge_part_1.toml",
        "data/weapon_merge_part_2.toml",
        "data/weapon_merge_part_3.toml",
        "data/weapon_merge_part_4.toml",
        ], "weapon", None).unwrap();

    dbg!("{}", c);
}