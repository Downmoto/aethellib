use aethellib::prelude::*;

fn main() {
    let c = Corpus::from_files(
        &[
            "data/weapon_merge_part_1.toml",
            "data/weapon_merge_part_2.toml",
            "data/weapon_merge_part_3.toml",
            "data/weapon_merge_part_4.toml",
        ],
        "weapon",
        None,
    )
    .expect("should read files and unwrap to Corpus");

    let c2 = Corpus::builder("weapon")
        .add_document(Document {
            source_id: "01".into(),
            source_hash: "hashatash".into(),
            source_path: "01.inline".into(),
            metadata: DocumentMetadata {
                title: String::from("inline document"),
                target: String::from("weapon"),
                desc: None,
                author: None,
                version: None,
                schema: None,
            },
            sections: Vec::<Section>::new(),
        })
        .build()
        .expect("should build");

    let c = c.combine(c2);
    dbg!(&c);

    let first_name_pool = c.pooled_values_for_field_section("prefix", "name").unwrap();

    dbg!(first_name_pool);
}
