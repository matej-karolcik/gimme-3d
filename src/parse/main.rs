use anyhow::anyhow;

fn main() {
    let gltf_dir = std::env::args().nth(1).expect("No gltf path given");

    let gltf_dir_path = std::path::Path::new(&gltf_dir);
    if !gltf_dir_path.exists() || !gltf_dir_path.is_dir() {
        panic!("gltf path does not exist or is not a directory");
    }

    gltf_dir_path.read_dir().unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            Some(path)
        })
        .for_each(|path| {
            let path_str = path.to_str().unwrap();
            if path.is_dir() {
                if let Err(e) = check_dir(path_str) {
                    println!("Error: {:?}", e);
                }
                return;
            } else {
                check_one(path_str).unwrap();
            }
        });
}

fn check_dir(dir: &str) -> anyhow::Result<()> {
    let gltf_dir_path = std::path::Path::new(dir);
    if !gltf_dir_path.exists() || !gltf_dir_path.is_dir() {
        return Err(anyhow!("gltf path does not exist or is not a directory"));
    }

    gltf_dir_path.read_dir().unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                Some(path)
            } else {
                None
            }
        })
        .for_each(|path| {
            let path_str = path.to_str().unwrap();
            if let Err(e) = check_one(path_str) {
                println!("Error: {:?}", e);
            }
        });

    Ok(())
}

fn check_one(gltf_path: &str) -> anyhow::Result<()> {
    if !gltf_path.ends_with(".gltf") {
        return Ok(());
    }

    println!("Checking: {}", gltf_path);

    let (doc, _, _) = gltf::import(gltf_path)?;
    let default_scene_maybe = doc.default_scene();

    if let None = default_scene_maybe {
        return Err(anyhow!("No default scene"));
    }

    let scene = default_scene_maybe.unwrap();

    let camera = gimme_the_3d::gltf::extract(&scene, gimme_the_3d::gltf::get_camera);
    if camera.is_none() {
        println!("No camera found");
    }
    let mesh = gimme_the_3d::gltf::extract(&scene, gimme_the_3d::gltf::get_mesh);
    if mesh.is_none() {
        println!("No mesh found");
    }

    Ok(())
}
