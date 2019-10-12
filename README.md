# YAGIBUU - Yet Another Global Illumination But Ultimately Ugly
YAGIBUU is an overly bootstrapped PBR renderer with a Voxel Cone Tracing implementation stuck inside of it, on in other words: held hostage by the dark powers of deadlines and ad-hoc improvised solutions.

This project is my thesis implementation, whose sole purpose is to compare two voxelization algorithms for achieving real-time GI through Cone Tracing.
Initially this project was using Rendy for the rendering, but a combination of unforeseen life events and mismanagement led to a major rewrite and an even shorter deadline.

This project was completely rewritten in a monstrous combination of raw OpenGL and a random helper crate which lacked needed features. Intuitively the code is a mess, an even uglier mess than the overly appropriate project's name.

```
"This is the ugliest thing I have ever seen."
                                                     - Yesterday's ugliest entity.
```

## Planned Features
- [x] Voxelization by Crassin & Green
- [ ] Voxelization by Rauwendaal
- [ ] Diffuse GI!
- [x] Major changes in the last second
- [x] Tears
- [x] Suffering
- [x] Is this hope??
- [x] Joy!

## Hopefully-one-day Features
- [ ] Throw it all away
- [ ] Rewrite it all in Vulkan and with a more reasonable deadline