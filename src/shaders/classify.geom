#version 450 core

#include <shared.glsl>

#define LARGE 0
#define SMALL 1

in VSOUT {
	vec3 w_position;
	vec3 w_normal;
	vec2 uv;
 	int  id;
} v_in[];

uniform int u_width;
uniform int u_height;
uniform int u_depth;

uniform mat4 pv;

out vec3 gw_position;
out vec3 gw_normal;
out vec2 g_uv;
out flat vec4 g_AABB;
out flat int g_swizzle;


layout(binding = 0, r32ui) uniform volatile coherent restrict uimage3D u_voxel_albedo;
layout(binding = 1, r32ui) uniform volatile coherent restrict uimage3D u_voxel_normal;
layout(binding = 2, r32ui) uniform volatile coherent restrict uimage3D u_voxel_emission;

layout(binding = 0) uniform sampler2D albedo_map;

layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

float triArea2D(vec2 v0, vec2 v1, vec2 v2) {
	return abs(v0.x * (v1.y - v2.y) + v1.x * (v2.y - v0.y) + v2.x * (v0.y - v1.y)) * 0.5;
}

//classify triangle as either LARGE or SMALL acorrding to the selected method
int classifyTriPostSwizzle(vec3 v0, vec3 v1, vec3 v2, float cutoff) {
	float val = triArea2D(v0.xy, v1.xy, v2.xy);
	return (val >= cutoff) ? LARGE : SMALL;
}

//Lookup up table of permutations matrices used to reverse swizzling
const mat3 unswizzleLUT[] = { mat3(0,1,0,0,0,1,1,0,0), mat3(0,0,1,1,0,0,0,1,0), mat3(1,0,0,0,1,0,0,0,1) };

//swizzle triangle vertices
void swizzleTri(inout vec3 v0, inout vec3 v1, inout vec3 v2, out vec3 n, out mat3 unswizzle) {
	n = cross(v1 - v0, v2 - v1);

	vec3 absN = abs(n);
	float maxAbsN = max(max(absN.x, absN.y), absN.z);

	if(absN.x >= absN.y && absN.x >= absN.z)			//X-direction dominant (YZ-plane)
	{													//Then you want to look down the X-direction
		v0.xyz = v0.yzx;
		v1.xyz = v1.yzx;
		v2.xyz = v2.yzx;

		n.xyz = n.yzx;

		//XYZ <-> YZX
		unswizzle = unswizzleLUT[0];
	}
	else if(absN.y >= absN.x && absN.y >= absN.z)		//Y-direction dominant (ZX-plane)
	{													//Then you want to look down the Y-direction
		v0.xyz = v0.zxy;
		v1.xyz = v1.zxy;
		v2.xyz = v2.zxy;

		n.xyz = n.zxy;

		//XYZ <-> ZXY
		unswizzle = unswizzleLUT[1];
	}
	else												//Z-direction dominant (XY-plane)
	{													//Then you want to look down the Z-direction (the default)
		v0.xyz = v0.xyz;
		v1.xyz = v1.xyz;
		v2.xyz = v2.xyz;

		n.xyz = n.xyz;

		//XYZ <-> XYZ
		unswizzle = unswizzleLUT[2];
	}
}

void voxelizeTriPostSwizzle(vec3 v0, vec3 v1, vec3 v2, vec3 n, mat3 unswizzle, ivec3 minVoxIndex, ivec3 maxVoxIndex)
{
	vec3 e0 = v1 - v0;	//figure 17/18 line 2
	vec3 e1 = v2 - v1;	//figure 17/18 line 2
	vec3 e2 = v0 - v2;	//figure 17/18 line 2

	//INward Facing edge normals XY
	vec2 n_e0_xy = (n.z >= 0) ? vec2(-e0.y, e0.x) : vec2(e0.y, -e0.x);	//figure 17/18 line 4
	vec2 n_e1_xy = (n.z >= 0) ? vec2(-e1.y, e1.x) : vec2(e1.y, -e1.x);	//figure 17/18 line 4
	vec2 n_e2_xy = (n.z >= 0) ? vec2(-e2.y, e2.x) : vec2(e2.y, -e2.x);	//figure 17/18 line 4

	//INward Facing edge normals YZ
	vec2 n_e0_yz = (n.x >= 0) ? vec2(-e0.z, e0.y) : vec2(e0.z, -e0.y);	//figure 17/18 line 5
	vec2 n_e1_yz = (n.x >= 0) ? vec2(-e1.z, e1.y) : vec2(e1.z, -e1.y);	//figure 17/18 line 5
	vec2 n_e2_yz = (n.x >= 0) ? vec2(-e2.z, e2.y) : vec2(e2.z, -e2.y);	//figure 17/18 line 5

	//INward Facing edge normals ZX
	vec2 n_e0_zx = (n.y >= 0) ? vec2(-e0.x, e0.z) : vec2(e0.x, -e0.z);	//figure 17/18 line 6
	vec2 n_e1_zx = (n.y >= 0) ? vec2(-e1.x, e1.z) : vec2(e1.x, -e1.z);	//figure 17/18 line 6
	vec2 n_e2_zx = (n.y >= 0) ? vec2(-e2.x, e2.z) : vec2(e2.x, -e2.z);	//figure 17/18 line 6

	float d_e0_xy = -dot(n_e0_xy, v0.xy) + max(0.0f, n_e0_xy.x) + max(0.0f, n_e0_xy.y);	//figure 17 line 7
	float d_e1_xy = -dot(n_e1_xy, v1.xy) + max(0.0f, n_e1_xy.x) + max(0.0f, n_e1_xy.y);	//figure 17 line 7
	float d_e2_xy = -dot(n_e2_xy, v2.xy) + max(0.0f, n_e2_xy.x) + max(0.0f, n_e2_xy.y);	//figure 17 line 7

	float d_e0_yz = -dot(n_e0_yz, v0.yz) + max(0.0f, n_e0_yz.x) + max(0.0f, n_e0_yz.y);	//figure 17 line 8
	float d_e1_yz = -dot(n_e1_yz, v1.yz) + max(0.0f, n_e1_yz.x) + max(0.0f, n_e1_yz.y);	//figure 17 line 8
	float d_e2_yz = -dot(n_e2_yz, v2.yz) + max(0.0f, n_e2_yz.x) + max(0.0f, n_e2_yz.y);	//figure 17 line 8

	float d_e0_zx = -dot(n_e0_zx, v0.zx) + max(0.0f, n_e0_zx.x) + max(0.0f, n_e0_zx.y);	//figure 18 line 9
	float d_e1_zx = -dot(n_e1_zx, v1.zx) + max(0.0f, n_e1_zx.x) + max(0.0f, n_e1_zx.y);	//figure 18 line 9
	float d_e2_zx = -dot(n_e2_zx, v2.zx) + max(0.0f, n_e2_zx.x) + max(0.0f, n_e2_zx.y);	//figure 18 line 9

	vec3 nProj = (n.z < 0.0) ? -n : n;	//figure 17/18 line 10

	const float dTri = dot(nProj, v0);
	const float dTriFatMin = dTri - max(nProj.x, 0) - max(nProj.y, 0);	//figure 17 line 11
	const float dTriFatMax = dTri - min(nProj.x, 0) - min(nProj.y, 0);	//figure 17 line 12

	const float nzInv = 1.0 / nProj.z;

	ivec3 p;					//voxel coordinate
	int   zMin,      zMax;		//voxel Z-range
	float zMinInt,   zMaxInt;	//voxel Z-intersection min/max
	float zMinFloor, zMaxCeil;	//voxel Z-intersection floor/ceil
	for(p.x = minVoxIndex.x; p.x < maxVoxIndex.x; p.x++)	//figure 17 line 13, figure 18 line 12
	{
		for(p.y = minVoxIndex.y; p.y < maxVoxIndex.y; p.y++)	//figure 17 line 14, figure 18 line 13
		{
			float dd_e0_xy = d_e0_xy + dot(n_e0_xy, p.xy);
			float dd_e1_xy = d_e1_xy + dot(n_e1_xy, p.xy);
			float dd_e2_xy = d_e2_xy + dot(n_e2_xy, p.xy);

			bool xy_overlap = (dd_e0_xy >= 0) && (dd_e1_xy >= 0) && (dd_e2_xy >= 0);

			if(xy_overlap)	//figure 17 line 15, figure 18 line 14
			{
				float dot_n_p = dot(nProj.xy, p.xy);
				zMinInt = (-dot_n_p + dTriFatMin) * nzInv;
				zMaxInt = (-dot_n_p + dTriFatMax) * nzInv;
				zMinFloor = floor(zMinInt);
				zMaxCeil  =  ceil(zMaxInt);

				zMin = int(zMinFloor) - int(zMinFloor == zMinInt);
				zMax = int(zMaxCeil ) + int(zMaxCeil  == zMaxInt);

				zMin = max(minVoxIndex.z, zMin);	//clamp to bounding box max Z
				zMax = min(maxVoxIndex.z, zMax);	//clamp to bounding box min Z

				for(p.z = zMin; p.z < zMax; p.z++)	//figure 17/18 line 18
				{
					float dd_e0_yz = d_e0_yz + dot(n_e0_yz, p.yz);
					float dd_e1_yz = d_e1_yz + dot(n_e1_yz, p.yz);
					float dd_e2_yz = d_e2_yz + dot(n_e2_yz, p.yz);

					float dd_e0_zx = d_e0_zx + dot(n_e0_zx, p.zx);
					float dd_e1_zx = d_e1_zx + dot(n_e1_zx, p.zx);
					float dd_e2_zx = d_e2_zx + dot(n_e2_zx, p.zx);

					bool yz_overlap = (dd_e0_yz >= 0) && (dd_e1_yz >= 0) && (dd_e2_yz >= 0);
					bool zx_overlap = (dd_e0_zx >= 0) && (dd_e1_zx >= 0) && (dd_e2_zx >= 0);

					if(yz_overlap && zx_overlap)	//figure 17/18 line 19
					{
						uvec4 val = uvec4(255);
						imageStore(u_voxel_albedo, ivec3(1, 1, 1), val);
						imageStore(u_voxel_normal, ivec3(unswizzle * p), val);
						imageStore(u_voxel_emission, ivec3(unswizzle * p), val);
					}
				}
				//z-loop
			}
			//xy-overlap test
		}
		//y-loop
	}
	//x-loop
}

void main() {
	vec3 n;
	mat3 swizzle;
	vec3 v0 = v_in[0].w_position;
	vec3 v1 = v_in[1].w_position;
	vec3 v2 = v_in[2].w_position;

	swizzleTri(v0, v1, v2, n, swizzle);

	int classification = classifyTriPostSwizzle(v0, v1, v2, 200090.2);
	imageStore(u_voxel_albedo, ivec3(1, 1, 1), uvec4(255));

	// if(classification == LARGE) {
		// int index = int(atomicCounterIncrement(largaaaeTriCount));

		// imageStore(largeIdx, 3*index+0, uvec4(In[0].vertexID));
		// imageStore(largeIdx, 3*index+1, uvec4(In[1].vertexID));
		// imageStore(largeIdx, 3*index+2, uvec4(In[2].vertexID));
	// } else if(classification == SMALL) {
		vec3 AABBmin = min(min(v0, v1), v2);
		vec3 AABBmax = max(max(v0, v1), v2);
		ivec3 volumeDim = ivec3(u_width, u_height, u_depth);

		ivec3 minVoxIndex = ivec3(clamp(floor(AABBmin), ivec3(0), volumeDim));
		ivec3 maxVoxIndex = ivec3(clamp( ceil(AABBmax), ivec3(0), volumeDim));

		voxelizeTriPostSwizzle(v0, v1, v2, n, swizzle, minVoxIndex, maxVoxIndex);
	// }

	EmitVertex();
	EmitVertex();
	EmitVertex();
	EndPrimitive();
//	memoryBarrier();

	// imageStore(largeIndirectElement, 0, uvec4(3*atomicCounter(largeTriCount)));
}