# Voxel Cone Tracing with Hybrid Voxelization
This is an overly bootstrapped PBR renderer with a Voxel Cone Tracing implementation stuck inside of it due tight deadlines.

That's also the implementation of my bachelor thesis, it is the comparison of two voxelization algorithms for Voxel Cone Tracing with and without native conservative rasterization. Those algorithms are the Per-Fragment Voxelization by [Crassin & Green](https://www.seas.upenn.edu/~pcozzi/OpenGLInsights/OpenGLInsights-SparseVoxelization.pdf) and Hybrid Voxelization by [Rauwendaal](http://jcgt.org/published/0002/01/02/paper-lowres.pdf)

This project was messily rewritten in a monstrous combination of raw OpenGL and a random helper crate which lacks necessary features. Intuitively the code is a mess, but this still works as an example implementation for Hybrid Voxelization, which relies heavily on OpenGL 4+ features, such as image load/store API, indirect rendering, atomic counters, atomic read/write to image buffers and on.

Regarding results, it was concluded that the Hybrid Voxelization is faster than the Per-Fragment Voxelization, but the Per-Fragment algorithm becomes the fastest combination when combined with hardware support for conservative rasterization using the `GL_NV_conservative_raster` [extension](https://www.khronos.org/registry/OpenGL/extensions/NV/NV_conservative_raster.txt). That's probably due the two-call overhead present in the Hybrid approach in contrast to the Per-Fragment approach decreased overhead by not having to dilate triangles in the geometry shader.
