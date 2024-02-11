<a href="https://www.rust-lang.org/" target="_blank" rel="noopener noreferrer"><img src="https://rustacean.net/assets/rustacean-orig-noshadow.svg" width="40" height="40"></a>
<a href="https://github.com/gfx-rs/wgpu" target="_blank" rel="noopener noreferrer"><img src="https://github.com/gfx-rs/wgpu/blob/trunk/logo.png" width="40" height="40"></a>

![gpl3](https://img.shields.io/badge/license-GPLv3-blue)
![last](https://img.shields.io/github/last-commit/NemoInfo/woxel)
![total](https://badgen.net/github/commits/NemoInfo/woxel)

# woxel
Web compatible voxel rendering engine. <br/>
The engine uses the VDB345[^1] data structure. It then performs HDDA[^3] in compute shaders to render the model.

![](photos/woxel_dragon.png) 

## Instalation
To run the project you will need the Rust nightly toolchain `1.75.0-nightly`[^2].
Easiest way to use the engine is with [cargo](https://doc.rust-lang.org/cargo/).<br/><br/>
For the **developer enviorment**, you can just run: 
``` shell
cargo run --release
```
<br/></br>
To build the **web enviorment**, you will need [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).</br>
You can then build the project, start a local server (doesn't matter how its served) and visit the `index.html` page.
```shell
wasm-pack build --target web
python3 -m http.server
```

## Use
You can load any `.vdb` model into the engine by adding it to the `assets/` folder.<br/> 
Then, on the dev pannel just select it from the dropdown menu. 

## Screenshots
![](photos/woxel_space_ray.png)
![](photos/woxel_space.png)


[^1]: [Ken Museth. 2013. VDB: High-resolution sparse volumes with dynamic topology](https://www.museth.org/Ken/Publications_files/Museth_TOG13.pdf)
[^2]: The VDB data structure is inherintly generic in shape. To achieve this in Rust I used the nightly [generic_const_expr](https://doc.rust-lang.org/beta/unstable-book/language-features/generic-const-exprs.html) feature
[^3]: [Ken Museth. 2014. Hierarchical digital differential analyzer for efficient ray-marching in OpenVDB.](https://www.museth.org/Ken/Publications_files/Museth_SIG14.pdf) 
