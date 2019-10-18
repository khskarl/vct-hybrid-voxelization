#version 450 core

layout (location = 0) in vec3 aPosition;
layout (location = 1) in vec2 aTexCoord;
layout (location = 2) in vec3 aNormal;

out VSOUT {
	vec3 w_position;
	vec3 w_normal;
	vec2 uv;
	int  id;
} v_out;

uniform int u_width;
uniform int u_height;
uniform int u_depth;

uniform mat4 pv;
uniform mat4 model;
layout(binding = 0, r32ui) uniform coherent uimage3D u_voxel_albedo;

void main() {
	vec4 w_position = pv * model * vec4(aPosition, 1.0);
	gl_Position = w_position;

	v_out.w_position = w_position.xyz;
	v_out.w_normal = normalize(vec3(model * vec4(aNormal, 1.0)));
	v_out.uv = aTexCoord;
	v_out.id = gl_VertexID;
	imageStore(u_voxel_albedo, ivec3(w_position.xyz), uvec4(255));
	imageStore(u_voxel_albedo, ivec3(0, 0, 0), uvec4(255));

}
