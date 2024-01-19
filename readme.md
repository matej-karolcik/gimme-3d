# Usage

The entrypoint to all functionality is the `cmd` bin target.

## Subcommands

- `serve`: start a server
- `render`: render a `glb/gltf` file or directory
- `collect`: collect all models from a local directory
- `convert`: convert `fbx` models into `gltf/glb`
- `download`: before starting a server, you can make the render requests a little faster
  by downloading the models to local path, the urls and local directory are configured using `config.toml`

```
Usage: cmd [COMMAND]

Commands:
  serve     Start http server (using config.toml for configuration)
  render    Render a single glb/gltf file or directory containing multiple
  collect   Collect model names from a local directory and save them in models.txt (for a later use in config.toml)
  download  Download models from a remote server to a local directory (for caching)
  convert   Convert fbx files into glb/gltf
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

# Api

### GET `/health`

For liveness and readiness probes.
Any status code other than 200 means the container should receive a backoff or a restart.

### POST `/render`

Endpoint for rendering a preview.
Request is in json format. [Example](request.json)

```json
{
    "model": "https://jq-staging-matko.s3.eu-central-1.amazonaws.com/gltf/1_p1_duvet-cover_1350x2000.glb",
    "texture_urls": [
        "https://images.unsplash.com/photo-1704372569833-c9f60f52eda3?q=80&w=3387&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D",
        "https://images.unsplash.com/photo-1704026438453-fde2ceb923ad?q=80&w=3436&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D",
        "https://images.unsplash.com/photo-1683009427037-c5afc2b8134d?q=80&w=3540&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDF8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D"
    ],
    "width": 2000,
    "height": 2000
}
```

### POST `/render-form`

Endpoint for rendering a preview.
The request is a post form, with following fields:

- `width` preview width
- `height` preview height
- `model` url of model to be used, the basename of the model will be used to look for a local file
- `textures` an array of textures in binary format, these will be applied to meshes
  in the same order as given here (`textures[0]`, `textures[1]`, ...)

# Configuration

Some features can be configured using the `config.toml` file.

- `port` local port for http server
- `models_base_url` the base url for downloading models
- `local_model_dir` local directory for where model files will be stored
- `models` a list of strings representing model filenames
  that will be appended to `models_base_url`

# Caveats

Some general advice:

- `three_d::HeadlessContext` is neither `Sync` nor `Send`,
  meaning it cannot be passed between threads. It will also panic if you try
  to create it in any other thread than the main one
  or if you attempt to create a second instance.
- Only serve one request at a time to avoid OOMs.
  The `tokio::sync::Semaphore` is used for this in the server.
- Rendering on MACOS and Unix like OSs is different in some respects,
  especially the Blend modes. 
