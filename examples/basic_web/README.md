# Iced Glass in the Browser

From initial testing, it is possible to run iced_glass in the browser. However, it is not stable across browsers, and is definitely considered an experimental feature. So far, only Google Chrome and Safari have been tested. In Chrome, the example works without issues. In Safari, the screen is rendered as fully black. However, the `iced` example "tour" also renders as black using the same settings in Safari, so the problem might not be due to adding COPY_SRC.

## A "bug" in `wgpu`

Due to the [implementation of webgpu in `wgpu`](https://github.com/gfx-rs/wgpu/blob/72bb53b0ed9c49b49f71d738cfe3acc982ce7ab0/wgpu/src/backend/webgpu.rs#L3941), only the `RENDER_ATTACHMENT` is advertised as a supported usage for the underlying texture:

```rust
// From webgpu.rs
wgt::SurfaceCapabilities {
    // https://gpuweb.github.io/gpuweb/#supported-context-formats
    formats,
    // Doesn't really have meaning on the web.
    present_modes: vec![wgt::PresentMode::Fifo],
    alpha_modes: vec![wgt::CompositeAlphaMode::Opaque],
    // Statically set to RENDER_ATTACHMENT for now. See https://gpuweb.github.io/gpuweb/#dom-gpucanvasconfiguration-usage
    usages: wgt::TextureUsages::RENDER_ATTACHMENT,
}
```

However, we are still able to force the usage by simply attaching it to the surface in `iced`:

```rust
surface.configure(
    &self.engine.device,
    &wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC,
        format: self.format,
        present_mode: self.settings.present_mode,
        width,
        height,
        alpha_mode: self.alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 1,
    },
);
```

Even if this isn't officially supported, it does work in Google Chrome. However, with the hard-coded surface usage from `wgpu`, there doesn't seem to be a way to query what usages the underlying surfaces _actually_ support. With this in mind, there are two main options:

- Force `COPY_SRC` regardless of what `wgpu` reports. This would be the most flexible and brittle option. It would enable compiling and running `iced_glass` in the browser, but it will cause rendering issues if the browser doesn't support `COPY_SRC` (e.g. Safari). Since there isn't a good way to detect the error, this option is unfortunately not great.
- Disable support for `iced_glass` in the browser completely. If `wgpu` can't provide accurate information regarding surface capabilities, we can't guess whether a feature is supported or not. The safer option is to not rely on `COPY_SRC` at all in the browser. Unfortunately, this means that browser support would be completely scrapped, since `iced_glass` relies on being able to copy the background texture ahead of its render pass.

### Possible workaround

There is another simple way to read the background texture. Instead of using `COPY_SRC`, `TEXTURE_BINDING` could be used to sample the background texture. However, that surface usage faces the exact same problems as `COPY_SRC`, since none of them are advertised as capabilities for web surfaces. Currently, no testing on using `TEXTURE_BINDING` has been done. From some cursory research on platform support for `COPY_SRC` and `TEXTURE_BINDING`, it seems that `COPY_SRC` should be more broadly supported.

## Installation

To compile for wasm, add the compilation target. Then use `trunk` to serve the application.

```
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Running

To start the server:

```sh
cd examples/basic_web
# add --release or --open if wanted
trunk serve
```