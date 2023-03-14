# InRidge

### Libraries

- [Docker](https://www.docker.com/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Bevy](https://github.com/bevyengine/bevy)
- [Hanabi](https://github.com/djeedai/bevy_hanabi)

### Development

```shell
cargo run
```

```shell
docker build . --target release-windows --output type=local,dest=release
docker build . --target release-wasm --output type=local,dest=release
```

### References

- [Hexsphere](https://devforum.roblox.com/t/hex-planets-dev-blog-i-generating-the-hex-sphere/769805) [hex tiling](https://www.redblobgames.com/x/1640-hexagon-tiling-of-sphere/) [HexTile orientation](https://gamedev.stackexchange.com/questions/125201/how-to-orient-a-hexagonal-tile-on-a-geodesic-sphere-goldberg-polyhedron)
- [Hex Borders](https://www.redblobgames.com/x/1541-hex-region-borders/)
- [Fixing Normal Map Issues](https://bgolus.medium.com/generating-perfect-normal-maps-for-unity-f929e673fc57#c508)
- [TBN Reconstruction](http://www.thetenthplanet.de/archives/1180)
- [Bounding Sphere](http://help.agi.com/AGIComponents/html/BlogBoundingSphere.htm)
- [GPU Hashes](https://www.shadertoy.com/view/XlGcRh) [Integer hash](https://nullprogram.com/blog/2018/07/31/)
- [Perlin/Simplex Noise](https://weber.itn.liu.se/~stegu/simplexnoise/simplexnoise.pdf) [Noise](https://github.com/Auburn/FastNoiseLite) [Perlin Noise Range](https://digitalfreepen.com/2017/06/20/range-perlin-noise.html)
- [spline mesh](https://www.youtube.com/watch?v=o9RK6O2kOKo) [bezier arc-length parameterization](https://gamedev.stackexchange.com/questions/5373/moving-ships-between-two-planets-along-a-bezier-missing-some-equations-for-acce/5427#5427)
- [State Machine Interruptions](https://arrowinmyknee.com/2020/10/16/interruption-to-self-in-unity/)
