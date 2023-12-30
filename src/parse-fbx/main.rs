use fbxcel::tree::any::AnyTree;

fn main() {
    let file = std::fs::File::open("assets/fbx/PhoneCase_IPhone12.fbx")
        .expect("Failed to open file");

    let reader = std::io::BufReader::new(file);

    let tree = AnyTree::from_seekable_reader(reader).expect("Failed to load tree");

    let (fbx_version, tree, footer) = match tree {
        AnyTree::V7400(fbx_version, tree, footer) => {
            Some((fbx_version, tree, footer))
        }
        _ => None,
    }.unwrap();

    let root = tree.root();
    let objects = root.children().find(|node| node.name() == "Objects").unwrap();

    root.children_by_name("Objects").for_each(|node| {
        node.children().for_each(|node| {
            if node.name() != "Model" {
                return;
            }

            node.attributes().iter().for_each(|attribute| {
                println!("Attribute: {:?}", attribute);
            });

            node.children().for_each(|node| {
                println!("  Child: {:?}", node.name());
                if node.name() == "Properties70" {
                    node.children().for_each(|node| {
                        println!("    Child: {:?}", node.name());
                        node.attributes().iter().for_each(|attribute| {
                            println!("    Attribute: {:?}", attribute);
                        });
                    });
                }
                node.attributes().iter().for_each(|attribute| {
                    println!("  Attribute: {:?}", attribute);
                });
            });
        });
    });
}
